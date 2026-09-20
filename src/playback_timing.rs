//! Low-overhead timing for the local playback startup path.
//!
//! The player, event receiver, and audio sink run on different threads. This
//! shared probe records only the first observation of each startup phase and
//! emits a compact summary once the replacement track reaches the audio queue.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

const DISPATCHED: u16 = 1 << 0;
const LOADING: u16 = 1 << 1;
const TRACK_CHANGED: u16 = 1 << 2;
const PLAYING: u16 = 1 << 3;
const SINK_START: u16 = 1 << 4;
const OUTPUT_OPENING: u16 = 1 << 5;
const OUTPUT_OPENED: u16 = 1 << 6;
const FIRST_AUDIO: u16 = 1 << 7;
const COMPLETE: u16 = TRACK_CHANGED | PLAYING | FIRST_AUDIO;

#[derive(Default)]
pub(crate) struct PlaybackTiming {
    next_id: AtomicU64,
    active: Mutex<Option<Trace>>,
}

struct Trace {
    id: u64,
    kind: String,
    started: Instant,
    last_observed: Instant,
    seen: u16,
    loading: Option<Duration>,
    track_changed: Option<Duration>,
    playing: Option<Duration>,
    output_opened: Option<Duration>,
    first_audio: Option<Duration>,
}

impl PlaybackTiming {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Starts a timeline before local interruption work and command dispatch.
    pub(crate) fn begin(&self, kind: &str, target: Option<&str>) {
        let now = Instant::now();
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let mut active = self.active.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(previous) = active.take() {
            log::info!(
                target: "magicspot::playback_timing",
                "playback timing #{} replaced after {} ms",
                previous.id,
                now.saturating_duration_since(previous.started).as_millis()
            );
        }
        *active = Some(Trace::new(id, kind, now));
        log::info!(
            target: "magicspot::playback_timing",
            "playback timing #{id}: request kind={kind} target={}",
            target.unwrap_or("unknown")
        );
    }

    pub(crate) fn dispatched(&self) {
        self.observe(DISPATCHED, "command dispatched", None, false);
    }

    pub(crate) fn loading(&self, uri: &str) {
        self.observe(LOADING, "librespot loading", Some(uri), true);
    }

    pub(crate) fn track_changed(&self, uri: &str) {
        self.observe(TRACK_CHANGED, "track metadata ready", Some(uri), true);
    }

    pub(crate) fn playing(&self, uri: &str) {
        self.observe(PLAYING, "librespot playing", Some(uri), false);
    }

    pub(crate) fn sink_start(&self) {
        self.observe(SINK_START, "audio sink start", None, false);
    }

    pub(crate) fn output_opening(&self) {
        self.observe(OUTPUT_OPENING, "audio output opening", None, false);
    }

    pub(crate) fn output_opened(&self) {
        self.observe(OUTPUT_OPENED, "audio output opened", None, false);
    }

    pub(crate) fn first_audio_queued(&self) {
        self.observe(FIRST_AUDIO, "first audio packet queued", None, false);
    }

    pub(crate) fn preloaded(&self, uri: &str) {
        log::info!(
            target: "magicspot::playback_timing",
            "playback preload ready: {uri}"
        );
    }

    pub(crate) fn failed(&self, reason: &str) {
        let now = Instant::now();
        let trace = self
            .active
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(trace) = trace {
            log::warn!(
                target: "magicspot::playback_timing",
                "playback timing #{} failed after {} ms: {reason}",
                trace.id,
                now.saturating_duration_since(trace.started).as_millis()
            );
        }
    }

    pub(crate) fn stopped(&self) {
        self.failed("playback stopped before first audio");
    }

    fn observe(&self, bit: u16, phase: &str, detail: Option<&str>, start_if_missing: bool) {
        let now = Instant::now();
        let mut active = self.active.lock().unwrap_or_else(PoisonError::into_inner);
        if active.is_none() {
            if !start_if_missing {
                return;
            }
            let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
            *active = Some(Trace::new(id, "spotify-event", now));
            log::info!(
                target: "magicspot::playback_timing",
                "playback timing #{id}: request kind=spotify-event target={}",
                detail.unwrap_or("unknown")
            );
        }
        let trace = active.as_mut().expect("an active trace was just created");
        if trace.seen & bit != 0 {
            return;
        }
        trace.seen |= bit;
        let total = now.saturating_duration_since(trace.started);
        let delta = now.saturating_duration_since(trace.last_observed);
        trace.last_observed = now;
        match bit {
            LOADING => trace.loading = Some(total),
            TRACK_CHANGED => trace.track_changed = Some(total),
            PLAYING => trace.playing = Some(total),
            OUTPUT_OPENED => trace.output_opened = Some(total),
            FIRST_AUDIO => trace.first_audio = Some(total),
            _ => {}
        }
        log::info!(
            target: "magicspot::playback_timing",
            "playback timing #{}: +{} ms (delta {} ms) {phase}{}",
            trace.id,
            total.as_millis(),
            delta.as_millis(),
            detail.map_or(String::new(), |value| format!(" {value}"))
        );

        if trace.seen & COMPLETE == COMPLETE {
            let trace = active.take().expect("the completed trace is active");
            log::info!(
                target: "magicspot::playback_timing",
                "playback timing #{} summary: kind={} loading={} track_ready={} playing={} output_open={} first_audio={} ms",
                trace.id,
                trace.kind,
                millis(trace.loading),
                millis(trace.track_changed),
                millis(trace.playing),
                millis(trace.output_opened),
                millis(trace.first_audio)
            );
        }
    }
}

impl Trace {
    fn new(id: u64, kind: &str, now: Instant) -> Self {
        Self {
            id,
            kind: kind.to_owned(),
            started: now,
            last_observed: now,
            seen: 0,
            loading: None,
            track_changed: None,
            playing: None,
            output_opened: None,
            first_audio: None,
        }
    }
}

fn millis(value: Option<Duration>) -> String {
    value.map_or_else(|| "n/a".into(), |value| value.as_millis().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_complete_timeline_is_released_for_the_next_track() {
        let timing = PlaybackTiming::new();
        timing.begin("next", None);
        timing.dispatched();
        timing.loading("spotify:track:first");
        timing.first_audio_queued();
        timing.track_changed("spotify:track:first");
        timing.playing("spotify:track:first");

        assert!(timing.active.lock().unwrap().is_none());

        timing.loading("spotify:track:second");
        let active = timing.active.lock().unwrap();
        assert_eq!(
            active.as_ref().map(|trace| trace.kind.as_str()),
            Some("spotify-event")
        );
    }

    #[test]
    fn repeated_phase_notifications_are_ignored() {
        let timing = PlaybackTiming::new();
        timing.begin("load", Some("spotify:track:test"));
        timing.loading("spotify:track:test");
        let first = timing
            .active
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .last_observed;
        timing.loading("spotify:track:test");
        let second = timing
            .active
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .last_observed;
        assert_eq!(first, second);
    }
}
