//! Album art: fetched once, kept on disk, decoded by egui on demand.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Instant, SystemTime};

use egui::load::{Bytes, BytesLoadResult, BytesLoader, BytesPoll, LoadError};
use sha1::{Digest, Sha1};

/// Maximum artwork bytes held in memory.
///
/// Time-based eviction does not work here: after creating a texture, egui no
/// longer requests its source bytes. Visible images were therefore evicted and
/// reloaded every two and a half minutes (#129).
///
/// Size-based eviction keeps visible images stable.
const HELD_BYTES: usize = 64 * 1024 * 1024;
const MAX_ART_BYTES: usize = 8 * 1024 * 1024;
const DISK_BYTES: u64 = 512 * 1024 * 1024;
const BLURRED_PREFIX: &str = "magicspot-blurred:";

type FetchResult = Result<Arc<[u8]>, String>;
type FetchFlight = tokio::sync::Mutex<Option<FetchResult>>;

#[derive(Default)]
struct DiskState {
    /// Scanned at startup; updated after each completed cache write.
    bytes: Option<u64>,
}

/// Decoded ColorImage plus the GPU texture, both RGBA.
fn decoded_and_texture_bytes(width: usize, height: usize) -> usize {
    2 * width.saturating_mul(height).saturating_mul(4)
}

enum Entry {
    Pending,
    Ready {
        bytes: Option<Arc<[u8]>>,
        last_used: Instant,
        /// JPEG bytes still held, plus decoded image and texture once painted.
        retained: usize,
    },
    Failed(String),
}

struct Inner {
    entries: Mutex<HashMap<String, Entry>>,
    flights: Mutex<HashMap<String, Weak<FetchFlight>>>,
    disk: Mutex<DiskState>,
    media_art_file: Mutex<Option<PathBuf>>,
    http: reqwest::Client,
    runtime: tokio::runtime::Handle,
    cache_dir: PathBuf,
}

#[derive(Clone)]
pub struct ArtLoader {
    inner: Arc<Inner>,
}

impl ArtLoader {
    pub fn new(http: reqwest::Client, runtime: tokio::runtime::Handle, cache_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&cache_dir);
        let inner = Arc::new(Inner {
            entries: Mutex::new(HashMap::new()),
            flights: Mutex::new(HashMap::new()),
            disk: Mutex::new(DiskState::default()),
            media_art_file: Mutex::new(None),
            http,
            runtime,
            cache_dir,
        });
        let cleanup = Arc::clone(&inner);
        inner.runtime.spawn_blocking(move || {
            if let Err(error) = cleanup.trim_disk_cache(DISK_BYTES, None) {
                log::debug!("artwork cache cleanup skipped: {error}");
            }
        });
        Self { inner }
    }

    /// Bytes for `url`, from memory, disk, or the network.
    pub async fn fetch(&self, url: &str) -> Result<Arc<[u8]>, String> {
        self.inner.fetch(url).await
    }

    /// A loader URI for a small, pre-blurred derivative of an artwork URL.
    pub fn blurred_uri(url: &str) -> String {
        format!("{BLURRED_PREFIX}{url}")
    }

    /// Marks artwork as visible so size-based eviction keeps it stable.
    pub fn touch(&self, url: &str) {
        if let Some(Entry::Ready { last_used, .. }) = self
            .inner
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get_mut(url)
        {
            *last_used = Instant::now();
        }
    }

    /// Evicts failed entries and the oldest artwork above the memory limit.
    pub fn evict(&self, ctx: &egui::Context) {
        let letting_go: Vec<String> = {
            let entries = self.inner.entries.lock().unwrap_or_else(|p| p.into_inner());
            let mut failed: Vec<String> = Vec::new();
            let mut held: Vec<(String, Instant, usize)> = Vec::new();
            for (url, entry) in entries.iter() {
                match entry {
                    // Forget failures so a later request can retry.
                    Entry::Failed(_) => failed.push(url.clone()),
                    Entry::Ready {
                        last_used,
                        retained,
                        ..
                    } => {
                        held.push((url.clone(), *last_used, *retained));
                    }
                    Entry::Pending => {}
                }
            }
            failed.extend(over_budget(held, HELD_BYTES));
            failed
        };
        for url in letting_go {
            ctx.forget_image(&url);
            self.forget(&url);
        }
    }

    /// The disk-cache file holding `url`'s artwork, once it has been fetched.
    ///
    /// The cache is written atomically (a `.part` file, then a rename), so a
    /// file that is here at all holds a complete, successful response. The
    /// desktop media controls hand this path to the platform instead of the
    /// remote URL: macOS loads cover art itself, synchronously, inside a
    /// callback that cannot report a failure.
    pub fn cached_file(&self, url: &str) -> Option<PathBuf> {
        let path = self.inner.cache_path(url);
        std::fs::metadata(&path)
            .is_ok_and(|meta| meta.is_file() && meta.len() > 0)
            .then_some(path)
    }

    /// Keep the file handed to desktop media controls through disk eviction.
    pub fn set_media_art_file(&self, file: Option<&Path>) {
        let mut current = self
            .inner
            .media_art_file
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if current.as_deref() != file {
            *current = file.map(Path::to_path_buf);
        }
    }

    /// Drops held JPEG bytes once egui has made a texture. The disk cache
    /// remains for later reloads.
    pub fn release_bytes(&self, url: &str) {
        self.inner.drop_bytes(url);
    }

    /// Record decoded image + texture size after egui has uploaded the cover.
    pub fn note_decoded(&self, url: &str, width: usize, height: usize) {
        if let Some(Entry::Ready {
            bytes, retained, ..
        }) = self
            .inner
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get_mut(url)
        {
            let jpeg = bytes.as_ref().map(|bytes| bytes.len()).unwrap_or(0);
            *retained = jpeg + decoded_and_texture_bytes(width, height);
        }
    }

    pub fn clear_disk_cache(&self) -> std::io::Result<u64> {
        let mut disk = self.inner.disk.lock().unwrap_or_else(|p| p.into_inner());
        let mut removed = 0;
        for entry in std::fs::read_dir(&self.inner.cache_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                removed += entry.metadata().map(|m| m.len()).unwrap_or(0);
                let _ = std::fs::remove_file(entry.path());
            }
        }
        disk.bytes = None;
        Ok(removed)
    }
}

/// Which artwork to let go of so that what is kept fits `budget`,
/// oldest first.
///
/// "Oldest" is when egui last needed the bytes, which for a picture it
/// has already made a texture of is when it first loaded. That makes
/// this a rough order rather than a true reading of what is on screen,
/// which is why the budget is generous: being roughly right about which
/// to drop only matters once there is far more artwork than any window
/// is showing.
fn over_budget(mut held: Vec<(String, Instant, usize)>, budget: usize) -> Vec<String> {
    let mut total: usize = held.iter().map(|(_, _, bytes)| bytes).sum();
    if total <= budget {
        return Vec::new();
    }
    held.sort_by_key(|(_, last_used, _)| *last_used);
    let mut letting_go = Vec::new();
    for (url, _, bytes) in held {
        if total <= budget {
            break;
        }
        total = total.saturating_sub(bytes);
        letting_go.push(url);
    }
    letting_go
}

impl Inner {
    fn cache_path(&self, url: &str) -> PathBuf {
        let digest = Sha1::digest(url.as_bytes());
        let mut name = String::with_capacity(40);
        for byte in digest {
            use std::fmt::Write;
            let _ = write!(name, "{byte:02x}");
        }
        self.cache_dir.join(name)
    }

    fn fetch_flight(&self, url: &str) -> Arc<FetchFlight> {
        let mut flights = self.flights.lock().unwrap_or_else(|p| p.into_inner());
        if flights.len() >= 1024 {
            flights.retain(|_, flight| flight.strong_count() > 0);
        }
        if let Some(flight) = flights.get(url).and_then(Weak::upgrade) {
            return flight;
        }
        let flight = Arc::new(FetchFlight::new(None));
        flights.insert(url.to_owned(), Arc::downgrade(&flight));
        flight
    }

    fn trim_disk_cache(&self, budget: u64, writing: Option<&Path>) -> std::io::Result<()> {
        let mut disk = self.disk.lock().unwrap_or_else(|p| p.into_inner());
        self.trim_disk_cache_locked(&mut disk, budget, writing)
    }

    fn trim_disk_cache_locked(
        &self,
        disk: &mut DiskState,
        budget: u64,
        writing: Option<&Path>,
    ) -> std::io::Result<()> {
        let mut files = Vec::new();
        let mut total = 0u64;
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.strip_suffix(".part").is_some_and(|stem| {
                stem.len() == 40 && stem.bytes().all(|byte| byte.is_ascii_hexdigit())
            }) {
                let _ = std::fs::remove_file(entry.path());
                continue;
            }
            if name.len() != 40 || !name.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                continue;
            }
            let metadata = entry.metadata()?;
            if !metadata.is_file() {
                continue;
            }
            total = total.saturating_add(metadata.len());
            files.push((
                entry.path(),
                metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                metadata.len(),
            ));
        }
        if total > budget {
            let mut protected = std::collections::HashSet::new();
            for (url, entry) in self
                .entries
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .iter()
            {
                if !matches!(entry, Entry::Failed(_)) {
                    protected
                        .insert(self.cache_path(url.strip_prefix(BLURRED_PREFIX).unwrap_or(url)));
                }
            }
            if let Some(path) = self
                .media_art_file
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .as_ref()
            {
                protected.insert(path.clone());
            }
            if let Some(path) = writing {
                protected.insert(path.to_path_buf());
            }
            files.sort_by_key(|(_, modified, _)| *modified);
            for (path, _, bytes) in files {
                if total <= budget {
                    break;
                }
                if !protected.contains(&path) && std::fs::remove_file(&path).is_ok() {
                    total = total.saturating_sub(bytes);
                }
            }
        }
        disk.bytes = Some(total);
        Ok(())
    }

    fn write_cache(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        let mut disk = self.disk.lock().unwrap_or_else(|p| p.into_inner());
        let previous = std::fs::metadata(path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        let temporary = path.with_extension("part");
        std::fs::write(&temporary, bytes)?;
        if let Err(error) = crate::util::replace_file(&temporary, path) {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }
        disk.bytes = disk.bytes.map(|total| {
            total
                .saturating_sub(previous)
                .saturating_add(bytes.len() as u64)
        });
        if disk.bytes.is_none_or(|total| total > DISK_BYTES) {
            self.trim_disk_cache_locked(&mut disk, DISK_BYTES, Some(path))?;
        }
        Ok(())
    }

    async fn fetch(self: &Arc<Self>, url: &str) -> Result<Arc<[u8]>, String> {
        if let Some(source) = url.strip_prefix(BLURRED_PREFIX) {
            let bytes = self.fetch_original(source).await?;
            return tokio::task::spawn_blocking(move || blurred_art(&bytes))
                .await
                .map_err(|error| error.to_string())?;
        }
        self.fetch_original(url).await
    }

    async fn fetch_original(self: &Arc<Self>, url: &str) -> Result<Arc<[u8]>, String> {
        if let Some(Entry::Ready {
            bytes: Some(bytes), ..
        }) = self
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(url)
        {
            return Ok(Arc::clone(bytes));
        }
        let flight = self.fetch_flight(url);
        let mut result = flight.lock().await;
        if let Some(cached) = result.as_ref() {
            return cached.clone();
        }
        let fetched = self.fetch_original_uncached(url).await;
        *result = Some(fetched.clone());
        fetched
    }

    async fn fetch_original_uncached(self: &Arc<Self>, url: &str) -> FetchResult {
        let path = self.cache_path(url);
        let cached = tokio::task::spawn_blocking({
            let path = path.clone();
            move || std::fs::read(path).ok()
        })
        .await
        .ok()
        .flatten();
        let bytes: Vec<u8> = match cached {
            Some(bytes) if !bytes.is_empty() && bytes.len() <= MAX_ART_BYTES => bytes,
            _ => {
                let response = self
                    .http
                    .get(url)
                    .send()
                    .await
                    .map_err(|error| error.to_string())?;
                if !response.status().is_success() {
                    return Err(format!("artwork request failed: {}", response.status()));
                }
                let bytes = response.bytes().await.map_err(|error| error.to_string())?;
                if bytes.len() > MAX_ART_BYTES {
                    return Err("artwork is too large".to_string());
                }
                let bytes = bytes.to_vec();
                let loader = Arc::clone(self);
                let payload = bytes.clone();
                if let Err(error) = self
                    .runtime
                    .spawn_blocking(move || loader.write_cache(&path, &payload))
                    .await
                    .map_err(|error| error.to_string())
                    .and_then(|result| result.map_err(|error| error.to_string()))
                {
                    log::debug!("unable to cache artwork: {error}");
                }
                bytes
            }
        };
        Ok(Arc::from(bytes))
    }

    fn start(self: &Arc<Self>, ctx: &egui::Context, url: String) {
        let loader = Arc::clone(self);
        let ctx = ctx.clone();
        self.runtime.spawn(async move {
            let result = loader.fetch(&url).await;
            let entry = match result {
                Ok(bytes) => Entry::Ready {
                    retained: bytes.len(),
                    bytes: Some(bytes),
                    last_used: Instant::now(),
                },
                Err(error) => Entry::Failed(error),
            };
            loader
                .entries
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .insert(url, entry);
            ctx.request_repaint();
        });
    }

    fn drop_bytes(&self, url: &str) {
        if let Some(Entry::Ready {
            bytes, retained, ..
        }) = self
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get_mut(url)
            && let Some(held) = bytes.take()
        {
            *retained = retained.saturating_sub(held.len());
        }
    }
}

impl BytesLoader for ArtLoader {
    fn id(&self) -> &'static str {
        "magicspot::ArtLoader"
    }

    fn load(&self, ctx: &egui::Context, uri: &str) -> BytesLoadResult {
        if !(uri.starts_with("https://")
            || uri.starts_with("http://")
            || uri.starts_with(BLURRED_PREFIX))
        {
            return Err(LoadError::NotSupported);
        }
        let mut entries = self.inner.entries.lock().unwrap_or_else(|p| p.into_inner());
        match entries.get_mut(uri) {
            Some(Entry::Ready {
                bytes: Some(bytes),
                last_used,
                ..
            }) => {
                *last_used = Instant::now();
                Ok(BytesPoll::Ready {
                    size: None,
                    bytes: Bytes::Shared(Arc::clone(bytes)),
                    mime: None,
                })
            }
            Some(Entry::Ready {
                bytes: None,
                last_used,
                ..
            }) => {
                *last_used = Instant::now();
                entries.insert(uri.to_string(), Entry::Pending);
                drop(entries);
                self.inner.start(ctx, uri.to_string());
                Ok(BytesPoll::Pending { size: None })
            }
            Some(Entry::Pending) => Ok(BytesPoll::Pending { size: None }),
            Some(Entry::Failed(error)) => Err(LoadError::Loading(error.clone())),
            None => {
                entries.insert(uri.to_string(), Entry::Pending);
                drop(entries);
                self.inner.start(ctx, uri.to_string());
                Ok(BytesPoll::Pending { size: None })
            }
        }
    }

    fn forget(&self, uri: &str) {
        self.inner
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(uri);
    }

    fn forget_all(&self) {
        self.inner
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
    }

    fn byte_size(&self) -> usize {
        self.inner
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .values()
            .map(|entry| match entry {
                Entry::Ready {
                    bytes: Some(bytes), ..
                } => bytes.len(),
                _ => 0,
            })
            .sum()
    }
}

fn blurred_art(bytes: &[u8]) -> Result<Arc<[u8]>, String> {
    let decoded = image::load_from_memory(bytes).map_err(|error| error.to_string())?;
    let softened = decoded
        .resize_to_fill(128, 128, image::imageops::FilterType::Triangle)
        .blur(10.0);
    let mut encoded = std::io::Cursor::new(Vec::new());
    softened
        .write_to(&mut encoded, image::ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    Ok(Arc::from(encoded.into_inner()))
}

/// A colour that represents an album cover, suitable for tinting a dark or
/// light surface: the most common saturated hue, with its lightness pulled
/// into a range that still reads as a background.
pub fn accent_color(bytes: &[u8]) -> Option<[u8; 3]> {
    let decoded = image::load_from_memory(bytes).ok()?;
    let small = decoded.thumbnail(48, 48).to_rgb8();
    let mut buckets: HashMap<(u8, u8, u8), (u64, [u64; 3])> = HashMap::new();
    for pixel in small.pixels() {
        let [r, g, b] = pixel.0;
        let (max, min) = (r.max(g).max(b) as f32, r.min(g).min(b) as f32);
        let saturation = if max == 0.0 { 0.0 } else { (max - min) / max };
        let lightness = (max + min) / 510.0;
        // Weight toward vivid mid-tones so black borders and white text lose.
        let weight = (1.0 + saturation * 6.0) * (1.0 - (lightness - 0.5).abs() * 1.4).max(0.05);
        let weight = (weight * 100.0) as u64;
        let key = (r >> 4, g >> 4, b >> 4);
        let bucket = buckets.entry(key).or_insert((0, [0, 0, 0]));
        bucket.0 += weight;
        bucket.1[0] += r as u64 * weight;
        bucket.1[1] += g as u64 * weight;
        bucket.1[2] += b as u64 * weight;
    }
    let (_, (weight, sum)) = buckets.into_iter().max_by_key(|(_, (weight, _))| *weight)?;
    if weight == 0 {
        return None;
    }
    Some([
        (sum[0] / weight) as u8,
        (sum[1] / weight) as u8,
        (sum[2] / weight) as u8,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn disk_cleanup_preserves_media_art_and_removes_partial_writes() {
        let dir = std::env::temp_dir().join(format!(
            "magicspot-art-disk-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let loader = ArtLoader::new(
            reqwest::Client::new(),
            runtime.handle().clone(),
            dir.clone(),
        );
        let paths: Vec<_> = (0..3)
            .map(|index| {
                loader
                    .inner
                    .cache_path(&format!("https://example.test/{index}"))
            })
            .collect();
        for path in &paths {
            std::fs::write(path, b"123456").unwrap();
        }
        let partial = paths[2].with_extension("part");
        std::fs::write(&partial, b"incomplete").unwrap();
        loader.set_media_art_file(Some(&paths[0]));

        loader.inner.trim_disk_cache(10, None).unwrap();

        assert!(paths[0].exists(), "the media-control image must remain");
        assert!(!partial.exists(), "unfinished writes must be removed");
        let remaining: u64 = paths
            .iter()
            .filter_map(|path| std::fs::metadata(path).ok())
            .map(|metadata| metadata.len())
            .sum();
        assert!(remaining <= 10, "cache stayed above its budget");
        drop(loader);
        drop(runtime);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_direct_fetches_share_one_request() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let hits = Arc::new(AtomicUsize::new(0));
        let done = Arc::new(AtomicBool::new(false));
        let server_hits = Arc::clone(&hits);
        let server_done = Arc::clone(&done);
        let server = std::thread::spawn(move || {
            while !server_done.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        server_hits.fetch_add(1, Ordering::Relaxed);
                        let mut request = [0; 1024];
                        let _ = stream.read(&mut request);
                        std::thread::sleep(Duration::from_millis(60));
                        let _ = stream.write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nART!",
                        );
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("artwork test server failed: {error}"),
                }
            }
        });
        let dir = std::env::temp_dir().join(format!(
            "magicspot-art-flight-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let loader = ArtLoader::new(
            reqwest::Client::new(),
            tokio::runtime::Handle::current(),
            dir.clone(),
        );
        let url = format!("http://{address}/cover");
        let barrier = Arc::new(tokio::sync::Barrier::new(9));
        let mut calls = tokio::task::JoinSet::new();
        for _ in 0..8 {
            let loader = loader.clone();
            let barrier = Arc::clone(&barrier);
            let url = url.clone();
            calls.spawn(async move {
                barrier.wait().await;
                loader.fetch(&url).await
            });
        }
        barrier.wait().await;
        while let Some(call) = calls.join_next().await {
            assert_eq!(&*call.unwrap().unwrap(), b"ART!");
        }
        done.store(true, Ordering::Relaxed);
        server.join().unwrap();
        assert_eq!(hits.load(Ordering::Relaxed), 1);
        assert!(loader.cached_file(&url).is_some());
        let _ = std::fs::remove_dir_all(dir);
    }

    /// The media controls ask for a file rather than a URL, and have to be
    /// told "not yet" rather than handed a path to nothing: macOS loads cover
    /// art itself and dereferences a failed load without checking it, which
    /// takes the whole process with it.
    #[test]
    fn a_cached_file_is_named_only_once_it_is_really_there() {
        let dir = std::env::temp_dir().join(format!("magicspot-art-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime to hand the loader");
        let loader = ArtLoader::new(
            reqwest::Client::new(),
            runtime.handle().clone(),
            dir.clone(),
        );
        let url = "https://i.scdn.co/image/abc";

        assert_eq!(loader.cached_file(url), None, "nothing downloaded yet");

        // A half-written download never appears under its real name -- the
        // cache renames one into place -- but an empty file is not artwork.
        let path = loader.inner.cache_path(url);
        std::fs::write(&path, b"").expect("an empty file");
        assert_eq!(loader.cached_file(url), None, "empty is not artwork");

        std::fs::write(&path, b"\xff\xd8\xff jpeg-ish").expect("a file with bytes");
        assert_eq!(loader.cached_file(url), Some(path));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn accent_color_finds_dominant_hue() {
        let mut image = image::RgbImage::new(16, 16);
        for (x, _, pixel) in image.enumerate_pixels_mut() {
            *pixel = if x < 12 {
                image::Rgb([20, 120, 200])
            } else {
                image::Rgb([255, 255, 255])
            };
        }
        let mut bytes = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
        let color = accent_color(&bytes).unwrap();
        assert!(
            color[2] > color[0],
            "expected the blue field, got {color:?}"
        );
    }

    #[test]
    fn blurred_art_is_a_small_reusable_png() {
        let source = image::DynamicImage::new_rgb8(24, 36);
        let mut encoded = std::io::Cursor::new(Vec::new());
        source
            .write_to(&mut encoded, image::ImageFormat::Png)
            .unwrap();

        let blurred = blurred_art(encoded.get_ref()).unwrap();
        let decoded = image::load_from_memory(&blurred).unwrap();

        assert_eq!((decoded.width(), decoded.height()), (128, 128));
    }

    fn held(items: &[(&str, u64, usize)]) -> Vec<(String, Instant, usize)> {
        let base = Instant::now();
        items
            .iter()
            .map(|(url, age_secs, bytes)| {
                (
                    (*url).to_string(),
                    base - std::time::Duration::from_secs(*age_secs),
                    *bytes,
                )
            })
            .collect()
    }

    /// Rule: nothing is let go of while it all fits. This is the case
    /// that matters: an evening of listening never reaches the budget,
    /// so no cover ever blinks out and back (#129).
    #[test]
    fn artwork_that_fits_is_all_kept() {
        let art = held(&[("a", 600, 1000), ("b", 300, 1000), ("c", 1, 1000)]);
        assert!(over_budget(art, 10_000).is_empty());
    }

    /// Rule: over the budget, the oldest go first, and only as many as
    /// it takes to fit.
    #[test]
    fn the_oldest_go_until_the_rest_fit() {
        let art = held(&[
            ("oldest", 900, 1000),
            ("middle", 600, 1000),
            ("newest", 1, 1000),
        ]);
        assert_eq!(over_budget(art, 2000), vec!["oldest"]);
    }

    #[test]
    fn enough_go_to_get_under_the_budget() {
        let art = held(&[
            ("oldest", 900, 1000),
            ("middle", 600, 1000),
            ("newest", 1, 1000),
        ]);
        assert_eq!(over_budget(art, 900), vec!["oldest", "middle", "newest"]);
    }

    /// Rule: an empty gallery asks nothing of anyone.
    #[test]
    fn nothing_held_lets_nothing_go() {
        assert!(over_budget(Vec::new(), 0).is_empty());
    }

    fn retained_total(loader: &ArtLoader) -> usize {
        loader
            .inner
            .entries
            .lock()
            .expect("lock")
            .values()
            .map(|entry| match entry {
                Entry::Ready { retained, .. } => *retained,
                _ => 0,
            })
            .sum()
    }

    #[test]
    fn large_covers_evict_using_decoded_and_texture_sizes() {
        let one = decoded_and_texture_bytes(640, 640);
        assert_eq!(one, 2 * 640 * 640 * 4);
        let jpeg = 50_000usize;
        let dir = std::env::temp_dir().join(format!(
            "magicspot-art-budget-{}-{}",
            std::process::id(),
            Instant::now().elapsed().as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("a runtime for eviction");
        let loader = ArtLoader::new(
            reqwest::Client::new(),
            runtime.handle().clone(),
            dir.clone(),
        );
        let now = Instant::now();
        for i in 0..40 {
            let url = format!("https://i.scdn.co/image/{i}");
            loader.inner.entries.lock().expect("lock").insert(
                url,
                Entry::Ready {
                    bytes: Some(Arc::from(vec![0u8; jpeg])),
                    last_used: now - Duration::from_secs(40 - i),
                    retained: jpeg,
                },
            );
        }
        let jpeg_total = retained_total(&loader);
        assert!(
            jpeg_total < HELD_BYTES,
            "JPEG-only covers must still fit the budget: {jpeg_total}"
        );
        for i in 0..40 {
            let url = format!("https://i.scdn.co/image/{i}");
            loader.release_bytes(&url);
            loader.note_decoded(&url, 640, 640);
        }
        let before = retained_total(&loader);
        assert_eq!(before, 40 * one);
        assert!(
            before > HELD_BYTES,
            "decoded 640×640 covers plus textures must exceed 64 MiB: {before}"
        );
        let ctx = egui::Context::default();
        loader.evict(&ctx);
        let after = retained_total(&loader);
        assert!(
            after <= HELD_BYTES,
            "eviction must bring retained decoded+texture bytes under budget: after={after}"
        );
        assert!(
            after < before,
            "a long scroll of large covers must free memory: before={before} after={after}"
        );
        let entries = loader.inner.entries.lock().expect("lock");
        assert!(
            !entries.contains_key("https://i.scdn.co/image/0"),
            "the oldest scrolled-away cover must go first"
        );
        assert!(
            entries.contains_key("https://i.scdn.co/image/39"),
            "the cover just scrolled into view must stay"
        );
        drop(entries);
        loader.forget_all();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn releasing_bytes_reloads_from_disk_off_the_ui_thread() {
        use egui::load::BytesLoader;
        use std::time::Duration as StdDuration;

        let dir = std::env::temp_dir().join(format!(
            "magicspot-art-reload-{}-{}",
            std::process::id(),
            Instant::now().elapsed().as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("a runtime for disk reload");
        let loader = ArtLoader::new(
            reqwest::Client::new(),
            runtime.handle().clone(),
            dir.clone(),
        );
        let url = "https://i.scdn.co/image/reload";
        let path = loader.inner.cache_path(url);
        std::fs::create_dir_all(path.parent().expect("cache dir")).expect("cache dir");
        std::fs::write(&path, b"\xff\xd8\xff jpeg-ish").expect("cached jpeg");
        loader.inner.entries.lock().expect("lock").insert(
            url.to_string(),
            Entry::Ready {
                bytes: None,
                last_used: Instant::now(),
                retained: 0,
            },
        );
        let ctx = egui::Context::default();
        let first = loader.load(&ctx, url).expect("load");
        assert!(
            matches!(first, BytesPoll::Pending { .. }),
            "disk reload must not block the UI thread"
        );
        let deadline = Instant::now() + StdDuration::from_secs(2);
        loop {
            std::thread::sleep(StdDuration::from_millis(20));
            match loader.load(&ctx, url) {
                Ok(BytesPoll::Ready { .. }) => break,
                Ok(BytesPoll::Pending { .. }) if Instant::now() < deadline => continue,
                _ => panic!("reload did not finish"),
            }
        }
        loader.forget_all();
        assert!(loader.inner.entries.lock().expect("lock").is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
