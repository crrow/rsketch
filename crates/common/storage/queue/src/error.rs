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

use std::io;
use std::path::PathBuf;

/// Queue operation errors.
#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    /// Filesystem I/O failure.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Failed to send message to IO worker.
    #[error("Channel send error")]
    ChannelSend,

    /// Failed to receive from IO worker.
    #[error("Channel receive error")]
    ChannelRecv,

    /// CRC mismatch detected during read.
    #[error("Corrupted message at sequence {0}")]
    CorruptedMessage(u64),

    /// Invalid or inaccessible file path.
    #[error("Invalid file path: {0}")]
    InvalidPath(PathBuf),

    /// Failed to roll to new data file.
    #[error("File rolling failed: {0}")]
    RollFileFailed(String),

    /// Memory mapping operation failed.
    #[error("Mmap operation failed: {0}")]
    MmapFailed(String),

    /// Index file read/write error.
    #[error("Index error: {0}")]
    IndexError(String),
}

/// Result type for queue operations.
pub type Result<T> = std::result::Result<T, QueueError>;
