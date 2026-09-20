//! Playback processing shared by every audio backend.

use librespot_playback::SAMPLE_RATE;
use librespot_playback::audio_backend::{Sink, SinkResult};
use librespot_playback::convert::Converter;
use librespot_playback::decoder::AudioPacket;
use librespot_playback::mixer::VolumeGetter;

/// Runs the equalizer, volume stage, and limiter before the real sink.
pub struct Processed {
    inner: Box<dyn Sink>,
    eq: crate::eq::Processor,
    volume: Box<dyn VolumeGetter + Send>,
    applies_volume: bool,
    limiter: crate::limiter::Limiter,
}

impl Processed {
    pub fn new(
        inner: Box<dyn Sink>,
        volume: Box<dyn VolumeGetter + Send>,
        applies_volume: bool,
        eq: crate::eq::SharedEq,
    ) -> Self {
        Self {
            inner,
            eq: crate::eq::Processor::new(eq),
            volume,
            applies_volume,
            limiter: crate::limiter::Limiter::new(f64::from(SAMPLE_RATE)),
        }
    }
}

fn full_scale(volume: f64, applied: bool) -> Option<f64> {
    if applied {
        Some(1.0)
    } else if volume > f64::EPSILON {
        Some(1.0 / volume)
    } else {
        None
    }
}

impl Sink for Processed {
    fn start(&mut self) -> SinkResult<()> {
        self.inner.start()
    }
    fn stop(&mut self) -> SinkResult<()> {
        self.inner.stop()
    }

    fn write(&mut self, packet: AudioPacket, converter: &mut Converter) -> SinkResult<()> {
        let packet = match packet {
            AudioPacket::Samples(mut samples) => {
                self.eq.process(&mut samples);
                let attenuation = self.volume.attenuation_factor();
                if self.applies_volume {
                    for sample in &mut samples {
                        *sample *= attenuation;
                    }
                }
                if let Some(full_scale) = full_scale(attenuation, self.applies_volume) {
                    self.limiter.process(&mut samples, full_scale);
                }
                AudioPacket::Samples(samples)
            }
            raw => raw,
        };
        self.inner.write(packet, converter)
    }
}

#[cfg(test)]
mod tests {
    use super::full_scale;

    #[test]
    fn full_scale_follows_the_volume_still_to_come() {
        assert_eq!(full_scale(0.5, true), Some(1.0));
        assert_eq!(full_scale(0.25, false), Some(4.0));
        assert_eq!(full_scale(1.0, false), Some(1.0));
        assert_eq!(full_scale(0.0, false), None);
    }
}
