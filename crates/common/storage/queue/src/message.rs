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

use bytes::Bytes;

#[derive(Debug)]
pub struct Message<'a> {
    pub sequence: u64,
    pub timestamp: u64,
    pub payload: &'a [u8],
}

#[derive(Debug, Clone)]
pub(crate) struct WriteEvent {
    pub sequence: u64,
    pub data: Bytes,
    pub timestamp: u64,
}

/// Message format: [length: 4 bytes][payload: variable][crc64: 8 bytes]
pub(crate) const MESSAGE_LENGTH_SIZE: usize = 4;
pub(crate) const MESSAGE_CRC_SIZE: usize = 8;
pub(crate) const MESSAGE_HEADER_SIZE: usize = MESSAGE_LENGTH_SIZE + MESSAGE_CRC_SIZE;

#[inline]
pub(crate) fn message_disk_size(payload_len: usize) -> usize {
    MESSAGE_LENGTH_SIZE + payload_len + MESSAGE_CRC_SIZE
}

pub(crate) fn timestamp_micros() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_disk_size() {
        assert_eq!(message_disk_size(0), 12);
        assert_eq!(message_disk_size(10), 22);
        assert_eq!(message_disk_size(100), 112);
    }

    #[test]
    fn test_timestamp_micros() {
        let ts1 = timestamp_micros();
        std::thread::sleep(std::time::Duration::from_micros(100));
        let ts2 = timestamp_micros();
        assert!(ts2 > ts1);
        assert!(ts2 - ts1 >= 100);
    }

    #[test]
    fn test_write_event_clone() {
        let event = WriteEvent {
            sequence: 42,
            data: Bytes::from("test data"),
            timestamp: 12345,
        };

        let cloned = event.clone();
        assert_eq!(cloned.sequence, 42);
        assert_eq!(cloned.data, Bytes::from("test data"));
        assert_eq!(cloned.timestamp, 12345);
    }
}
