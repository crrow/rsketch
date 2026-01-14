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

#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Channel send error")]
    ChannelSend,

    #[error("Channel receive error")]
    ChannelRecv,

    #[error("Corrupted message at sequence {0}")]
    CorruptedMessage(u64),

    #[error("Invalid file path: {0}")]
    InvalidPath(PathBuf),

    #[error("File rolling failed: {0}")]
    RollFileFailed(String),

    #[error("Mmap operation failed: {0}")]
    MmapFailed(String),

    #[error("Index error: {0}")]
    IndexError(String),
}

pub type Result<T> = std::result::Result<T, QueueError>;
