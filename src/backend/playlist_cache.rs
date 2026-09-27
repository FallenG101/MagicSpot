//! Incremental, crash-safe playlist checkpoints.
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::CachedPlaylist;
use crate::api::models::PlaylistItem;
use crate::model::PlaylistCache;
use serde::{Deserialize, Serialize};

static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Serialize, Deserialize)]
struct Manifest {
    version: u8,
    snapshot: String,
    data_file: String,
    bytes: u64,
    rows: usize,
    total: u32,
    next_offset: Option<u32>,
}

fn row_path(path: &Path, name: &str) -> io::Result<PathBuf> {
    let file = Path::new(name);
    if file.components().count() != 1 || file.file_name().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid row filename",
        ));
    }
    Ok(path.with_file_name(name))
}

fn manifest(path: &Path) -> io::Result<Manifest> {
    let value: Manifest = serde_json::from_reader(BufReader::new(File::open(path)?))?;
    if value.version != 2 || value.rows > value.total as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid playlist manifest",
        ));
    }
    row_path(path, &value.data_file)?;
    Ok(value)
}

pub(super) fn read(path: &Path) -> io::Result<PlaylistCache> {
    if let Ok(meta) = manifest(path) {
        let file = File::open(row_path(path, &meta.data_file)?)?;
        if file.metadata()?.len() < meta.bytes {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "short playlist rows",
            ));
        }
        let mut reader = BufReader::new(file).take(meta.bytes);
        let mut items = Vec::with_capacity(meta.rows);
        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            let mut block: Vec<PlaylistItem> = serde_json::from_str(&line)?;
            items.append(&mut block);
            line.clear();
        }
        if items.len() != meta.rows || meta.next_offset.is_some_and(|offset| offset > meta.total) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid playlist rows",
            ));
        }
        return Ok(PlaylistCache {
            snapshot: meta.snapshot,
            items,
            total: meta.total,
            next_offset: meta.next_offset,
        });
    }
    // Version 1 stored the whole playlist in one JSON file.
    let cached: CachedPlaylist = serde_json::from_reader(BufReader::new(File::open(path)?))?;
    let total = cached
        .total
        .unwrap_or_else(|| cached.items.len().try_into().unwrap_or(u32::MAX));
    if cached.items.len() > total as usize
        || cached.next_offset.is_some_and(|offset| offset > total)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid legacy playlist",
        ));
    }
    Ok(PlaylistCache {
        snapshot: cached.snapshot,
        items: cached.items,
        total,
        next_offset: cached.next_offset,
    })
}

fn publish(path: &Path, value: &Manifest) -> io::Result<()> {
    let temporary = path.with_extension("json.tmp");
    let result = (|| {
        let mut writer = BufWriter::new(File::create(&temporary)?);
        serde_json::to_writer(&mut writer, value)?;
        writer.flush()?;
        writer
            .into_inner()
            .map_err(|error| error.into_error())?
            .sync_all()?;
        crate::util::replace_file(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn append_block(file: File, rows: &[PlaylistItem]) -> io::Result<u64> {
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, rows)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    let file = writer.into_inner().map_err(|error| error.into_error())?;
    file.sync_all()?;
    Ok(file.metadata()?.len())
}

fn clean_orphaned_rows(path: &Path, keep: &str) -> io::Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    let Some(stem) = path.file_stem() else {
        return Ok(());
    };
    let prefix = format!("{}.rows-", stem.to_string_lossy());
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&prefix) && name != keep && entry.file_type()?.is_file() {
            let _ = fs::remove_file(entry.path());
        }
    }
    Ok(())
}

pub(super) fn write(path: &Path, cached: &CachedPlaylist) -> io::Result<()> {
    let total = cached.total.unwrap_or(cached.items.len() as u32);
    if cached.items.len() > total as usize
        || cached.next_offset.is_some_and(|offset| offset > total)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid playlist checkpoint",
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let old = manifest(path).ok();
    if let Some(mut meta) = old
        .as_ref()
        .filter(|meta| {
            meta.snapshot == cached.snapshot
                && meta.total == total
                && meta.rows <= cached.items.len()
                && meta.next_offset.unwrap_or(meta.total) <= cached.next_offset.unwrap_or(total)
        })
        .cloned()
    {
        let data = row_path(path, &meta.data_file)?;
        if let Ok(mut file) = OpenOptions::new().read(true).write(true).open(&data)
            && file.metadata()?.len() >= meta.bytes
        {
            file.set_len(meta.bytes)?;
            file.seek(SeekFrom::Start(meta.bytes))?;
            if meta.rows < cached.items.len() {
                meta.bytes = append_block(file, &cached.items[meta.rows..])?;
            }
            meta.rows = cached.items.len();
            meta.next_offset = cached.next_offset;
            publish(path, &meta)?;
            let _ = clean_orphaned_rows(path, &meta.data_file);
            return Ok(());
        }
    }
    let nonce = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let data = path.with_extension(format!("rows-{stamp:x}-{nonce:x}"));
    let data_file = data.file_name().unwrap().to_string_lossy().into_owned();
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&data)?;
    let result = (|| {
        let bytes = append_block(file, &cached.items)?;
        publish(
            path,
            &Manifest {
                version: 2,
                snapshot: cached.snapshot.clone(),
                data_file,
                bytes,
                rows: cached.items.len(),
                total,
                next_offset: cached.next_offset,
            },
        )
    })();
    if result.is_err() {
        let _ = fs::remove_file(&data);
    } else {
        let _ = clean_orphaned_rows(path, data.file_name().unwrap().to_string_lossy().as_ref());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn appends_only_new_rows_and_reads_legacy_cache() {
        let root = std::env::temp_dir().join(format!(
            "magicspot-incremental-cache-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("playlist.json");
        let mut cached = CachedPlaylist {
            snapshot: "first".into(),
            items: vec![PlaylistItem::default()],
            total: Some(3),
            next_offset: Some(1),
        };
        super::super::write_cached_playlist(&path, &cached).unwrap();
        assert_eq!(read(&path).unwrap().items.len(), 1);
        write(&path, &cached).unwrap();
        let before = manifest(&path).unwrap();
        let data = row_path(&path, &before.data_file).unwrap();
        // Bytes written before a failed manifest publication are ignored.
        OpenOptions::new()
            .append(true)
            .open(&data)
            .unwrap()
            .write_all(b"orphan\n")
            .unwrap();
        assert_eq!(read(&path).unwrap().items.len(), 1);
        cached.items.push(PlaylistItem::default());
        cached.next_offset = Some(2);
        write(&path, &cached).unwrap();
        let after = manifest(&path).unwrap();
        assert_eq!(before.data_file, after.data_file);
        assert!(after.bytes > before.bytes);
        assert_eq!(read(&path).unwrap().items.len(), 2);
        cached.snapshot = "replacement".into();
        write(&path, &cached).unwrap();
        assert!(!data.exists());
        assert_eq!(read(&path).unwrap().snapshot, "replacement");
        let _ = fs::remove_dir_all(root);
    }
}
