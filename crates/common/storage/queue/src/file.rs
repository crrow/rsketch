// Copyright 2025 Crrow
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Memory-mapped data file operations using mmap-io.

use crate::{FlushMode, QueueError, Result};
use mmap_io::MemoryMappedFile;
use std::path::{Path, PathBuf};

/// Memory-mapped data file for append-only writes.
///
/// Wraps mmap-io's MemoryMappedFile to provide a simpler interface
/// for the queue's append-only write pattern.
pub struct DataFile {
    mmap: MemoryMappedFile,
    path: PathBuf,
    size: u64,
}

impl DataFile {
    /// Create a new data file with pre-allocation.
    pub fn create<P: AsRef<Path>>(path: P, size: u64) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create memory-mapped file with pre-allocated size
        let mmap = MemoryMappedFile::create_rw(&path, size)
            .map_err(|e| QueueError::MmapFailed(e.to_string()))?;

        Ok(Self { mmap, path, size })
    }

    /// Open an existing data file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let mmap =
            MemoryMappedFile::open_rw(&path).map_err(|e| QueueError::MmapFailed(e.to_string()))?;

        let size = mmap.len();

        Ok(Self { mmap, path, size })
    }

    /// Write data at the specified offset.
    #[inline]
    pub fn write_at(&self, offset: u64, data: &[u8]) -> Result<()> {
        self.mmap
            .update_region(offset, data)
            .map_err(|e| QueueError::MmapFailed(e.to_string()))
    }

    /// Read data from the specified offset into the provided buffer.
    #[inline]
    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<()> {
        self.mmap
            .read_into(offset, buf)
            .map_err(|e| QueueError::MmapFailed(e.to_string()))
    }

    /// Get file size.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Flush data to disk based on flush mode.
    pub fn flush(&self, mode: FlushMode) -> Result<()> {
        match mode {
            FlushMode::Async => {
                // mmap-io uses async flush internally when appropriate
                self.mmap
                    .flush()
                    .map_err(|e| QueueError::MmapFailed(e.to_string()))?;
            }
            FlushMode::Sync => {
                self.mmap
                    .flush()
                    .map_err(|e| QueueError::MmapFailed(e.to_string()))?;
            }
            FlushMode::Batch { .. } => {
                // Batch mode is handled by caller, use regular flush
                self.mmap
                    .flush()
                    .map_err(|e| QueueError::MmapFailed(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Flush a specific range to disk.
    pub fn flush_range(&self, offset: u64, len: u64) -> Result<()> {
        self.mmap
            .flush_range(offset, len)
            .map_err(|e| QueueError::MmapFailed(e.to_string()))
    }
}

/// Read-only memory-mapped data file.
pub struct ReadOnlyDataFile {
    mmap: MemoryMappedFile,
    size: u64,
}

impl ReadOnlyDataFile {
    /// Open an existing data file in read-only mode.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mmap = MemoryMappedFile::open_ro(path.as_ref())
            .map_err(|e| QueueError::MmapFailed(e.to_string()))?;

        let size = mmap.len();

        Ok(Self { mmap, size })
    }

    /// Read data from the specified offset into the provided buffer.
    #[inline]
    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<()> {
        self.mmap
            .read_into(offset, buf)
            .map_err(|e| QueueError::MmapFailed(e.to_string()))
    }

    /// Get a slice of data at the specified offset.
    ///
    /// This is zero-copy for read-only mappings.
    #[inline]
    pub fn as_slice(&self, offset: u64, len: u64) -> Result<&[u8]> {
        self.mmap
            .as_slice(offset, len)
            .map_err(|e| QueueError::MmapFailed(e.to_string()))
    }

    /// Get file size.
    pub fn size(&self) -> u64 {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_data_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.data");
        let size = 4096;

        let file = DataFile::create(&path, size).unwrap();
        assert_eq!(file.size(), size);
        assert!(path.exists());
    }

    #[test]
    fn test_write_and_read_data_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.data");
        let size = 4096;

        // Create and write
        {
            let file = DataFile::create(&path, size).unwrap();
            let data = b"Hello, World!";
            file.write_at(0, data).unwrap();
            file.flush(FlushMode::Sync).unwrap();
        }

        // Read back
        {
            let file = ReadOnlyDataFile::open(&path).unwrap();
            let data = file.as_slice(0, 13).unwrap();
            assert_eq!(data, b"Hello, World!");
        }
    }

    #[test]
    fn test_read_into_buffer() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.data");
        let size = 4096;

        // Create and write
        {
            let file = DataFile::create(&path, size).unwrap();
            file.write_at(100, b"Test data at offset").unwrap();
            file.flush(FlushMode::Sync).unwrap();
        }

        // Read back using read_at
        {
            let file = ReadOnlyDataFile::open(&path).unwrap();
            let mut buf = [0u8; 19];
            file.read_at(100, &mut buf).unwrap();
            assert_eq!(&buf, b"Test data at offset");
        }
    }

    #[test]
    fn test_open_existing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.data");

        // Create file first
        {
            DataFile::create(&path, 1024).unwrap();
        }

        // Open existing
        let file = DataFile::open(&path).unwrap();
        assert_eq!(file.size(), 1024);
    }
}
