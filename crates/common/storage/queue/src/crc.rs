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

use crc::{CRC_64_ECMA_182, Crc};

pub(crate) const CRC64: Crc<u64> = Crc::<u64>::new(&CRC_64_ECMA_182);

#[inline]
pub(crate) fn calculate_crc(data: &[u8]) -> u64 {
    CRC64.checksum(data)
}

#[inline]
pub(crate) fn calculate_message_crc(length: u32, data: &[u8]) -> u64 {
    let mut digest = CRC64.digest();
    digest.update(&length.to_le_bytes());
    digest.update(data);
    digest.finalize()
}

#[inline]
pub(crate) fn verify_crc(data: &[u8], expected: u64) -> bool {
    calculate_crc(data) == expected
}

#[inline]
pub(crate) fn verify_message_crc(length: u32, data: &[u8], expected: u64) -> bool {
    calculate_message_crc(length, data) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_crc() {
        let data = b"Hello, World!";
        let crc1 = calculate_crc(data);
        let crc2 = calculate_crc(data);
        assert_eq!(crc1, crc2);

        let data2 = b"Hello, World?";
        let crc3 = calculate_crc(data2);
        assert_ne!(crc1, crc3);
    }

    #[test]
    fn test_calculate_message_crc() {
        let data = b"test message";
        let length = data.len() as u32;
        let crc = calculate_message_crc(length, data);

        let crc2 = calculate_message_crc(length, data);
        assert_eq!(crc, crc2);

        let crc3 = calculate_message_crc(length + 1, data);
        assert_ne!(crc, crc3);
    }

    #[test]
    fn test_verify_crc() {
        let data = b"verify me";
        let crc = calculate_crc(data);

        assert!(verify_crc(data, crc));
        assert!(!verify_crc(data, crc + 1));
        assert!(!verify_crc(b"wrong data", crc));
    }

    #[test]
    fn test_verify_message_crc() {
        let data = b"message to verify";
        let length = data.len() as u32;
        let crc = calculate_message_crc(length, data);

        assert!(verify_message_crc(length, data, crc));
        assert!(!verify_message_crc(length, data, crc + 1));
        assert!(!verify_message_crc(length + 1, data, crc));
        assert!(!verify_message_crc(length, b"wrong", crc));
    }
}
