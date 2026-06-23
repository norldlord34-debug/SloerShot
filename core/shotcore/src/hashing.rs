//! File/content hashing for the Hash Checker tool: MD5, SHA-1, SHA-256, SHA-512, CRC32.
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

/// Hash `data` with the named algorithm (md5/sha1/sha256/sha512/crc32). Defaults to sha256.
pub fn hash_bytes(data: &[u8], algo: &str) -> String {
 match algo.to_ascii_lowercase().as_str() {
 "md5" => hex(Md5::digest(data).as_slice()),
 "sha1" => hex(Sha1::digest(data).as_slice()),
 "sha512" => hex(Sha512::digest(data).as_slice()),
 "crc32" => format!("{:08x}", crc32(data)),
 _ => hex(Sha256::digest(data).as_slice()),
 }
}

fn hex(bytes: &[u8]) -> String {
 let mut s = String::with_capacity(bytes.len() * 2);
 for b in bytes {
 s.push_str(&format!("{:02x}", b));
 }
 s
}

/// Standard CRC-32 (IEEE 802.3, polynomial 0xEDB88320).
pub fn crc32(data: &[u8]) -> u32 {
 let mut crc: u32 = 0xFFFF_FFFF;
 for &byte in data {
 crc ^= byte as u32;
 for _ in 0..8 {
 if crc & 1 != 0 {
 crc = (crc >> 1) ^ 0xEDB8_8320;
 } else {
 crc >>= 1;
 }
 }
 }
 !crc
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn known_hashes_of_abc() {
 let d = b"abc";
 assert_eq!(hash_bytes(d, "md5"), "900150983cd24fb0d6963f7d28e17f72");
 assert_eq!(hash_bytes(d, "sha1"), "a9993e364706816aba3e25717850c26c9cd0d89d");
 assert_eq!(hash_bytes(d, "sha256"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
 assert_eq!(hash_bytes(d, "crc32"), "352441c2");
 }
 #[test]
 fn empty_string_hashes() {
 assert_eq!(hash_bytes(b"", "md5"), "d41d8cd98f00b204e9800998ecf8427e");
 assert_eq!(hash_bytes(b"", "crc32"), "00000000");
 }
}
