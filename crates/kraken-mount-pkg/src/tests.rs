//! Small synthetic host fixtures for the FIH/CNT boundary.
//! No copyrighted game data is required; these only exercise header/container parsing.
//! A successful header test does NOT mean an entire FPKG or the PS5 runtime was tested.

use alloc::{vec, vec::Vec};

use crate::{pkg, Error, PRead};

const PKG_SIZE: usize = 0x30000;
const CNT_AT: usize = 0x20000;
const CNT_TABLE_AT: usize = CNT_AT + 0x1000;

struct MemoryIo {
    bytes: Vec<u8>,
    max_read: usize,
    fail_from: Option<u64>,
}

impl MemoryIo {
    fn new(bytes: Vec<u8>) -> Self {
        Self { bytes, max_read: usize::MAX, fail_from: None }
    }

    fn chunked(mut self, max_read: usize) -> Self {
        self.max_read = max_read;
        self
    }
}

impl PRead for MemoryIo {
    fn pread(&self, off: u64, dst: &mut [u8]) -> Result<usize, Error> {
        if self.fail_from.is_some_and(|at| off >= at) {
            return Err(Error::Io(5));
        }
        let Ok(at) = usize::try_from(off) else { return Ok(0) };
        let n = self.bytes.len().saturating_sub(at).min(dst.len()).min(self.max_read);
        if n != 0 {
            dst[..n].copy_from_slice(&self.bytes[at..at + n]);
        }
        Ok(n)
    }
}

fn le64_at(out: &mut [u8], at: usize, v: u64) {
    out[at..at + 8].copy_from_slice(&v.to_le_bytes());
}

fn be32_at(out: &mut [u8], at: usize, v: u32) {
    out[at..at + 4].copy_from_slice(&v.to_be_bytes());
}

/// Enough bytes to validate FIH bounds without constructing the PFS contents.
fn fih() -> Vec<u8> {
    let mut out = vec![0u8; PKG_SIZE];
    out[..4].copy_from_slice(b"\x7fFIH");
    le64_at(&mut out, 0x10, pkg::BLOCK);
    le64_at(&mut out, 0x18, pkg::BLOCK);
    le64_at(&mut out, 0x20, pkg::BLOCK);
    le64_at(&mut out, 0x58, CNT_AT as u64);
    le64_at(&mut out, 0x60, pkg::BLOCK);
    out
}

fn with_cnt(mut bytes: Vec<u8>) -> Vec<u8> {
    bytes[CNT_AT..CNT_AT + 4].copy_from_slice(b"\x7fCNT");
    bytes[CNT_AT + 0x40..CNT_AT + 0x48].copy_from_slice(b"TEST-ID\0");
    be32_at(&mut bytes, CNT_AT + 0x10, 2);
    be32_at(&mut bytes, CNT_AT + 0x18, 0x1000);
    be32_at(&mut bytes, CNT_TABLE_AT, 0x1200);
    be32_at(&mut bytes, CNT_TABLE_AT + 16, 0x1100);
    be32_at(&mut bytes, CNT_TABLE_AT + 20, 48);
    be32_at(&mut bytes, CNT_TABLE_AT + 0x20, 0x2000);
    be32_at(&mut bytes, CNT_TABLE_AT + 0x20 + 8, 0x8000_0000);
    be32_at(&mut bytes, CNT_TABLE_AT + 0x20 + 16, 0x1200);
    be32_at(&mut bytes, CNT_TABLE_AT + 0x20 + 20, 128);
    bytes
}

#[test]
fn exact_read_handles_short_positional_reads() {
    let io = MemoryIo::new(vec![10, 20, 30, 40, 50, 60]).chunked(2);
    let mut out = [0u8; 4];
    pkg::pread_exact(&io, 1, &mut out).unwrap();
    assert_eq!(out, [20, 30, 40, 50]);
}

#[test]
fn exact_read_reports_truncation_instead_of_silent_short_read() {
    let io = MemoryIo::new(vec![1, 2]);
    let mut out = [0u8; 3];
    assert!(matches!(pkg::pread_exact(&io, 0, &mut out), Err(Error::Corrupt(_))));
}

#[test]
fn exact_read_propagates_io_error() {
    let io = MemoryIo { bytes: vec![1, 2, 3], max_read: 1, fail_from: Some(1) };
    let mut out = [0u8; 3];
    assert_eq!(pkg::pread_exact(&io, 0, &mut out), Err(Error::Io(5)));
}

#[test]
fn valid_fih_header_parses_with_fragmented_reads() {
    let io = MemoryIo::new(fih()).chunked(7);
    let header = pkg::header(&io, PKG_SIZE as u64).unwrap();
    assert_eq!(header.pfs_offset, pkg::BLOCK);
    assert_eq!(header.outer_blocks, 1);
    assert_eq!(header.sb_index, 0);
    assert_eq!(header.cnt_offset, CNT_AT as u64);
    assert_eq!(header.meta_base, 0);
}

#[test]
fn fpkgs_shorter_than_header_are_rejected() {
    let io = MemoryIo::new(vec![0; 64]);
    assert!(matches!(pkg::header(&io, 64), Err(Error::NotPs5(_))));
}

#[test]
fn declared_length_does_not_hide_truncated_physical_reads() {
    let io = MemoryIo::new(fih()[..12].to_vec());
    assert!(matches!(pkg::header(&io, PKG_SIZE as u64), Err(Error::Corrupt(_))));
}

#[test]
fn li_h_patch_and_ps4_package_have_distinct_errors() {
    let mut data = fih();
    data[..4].copy_from_slice(b"\x7fLIH");
    assert!(matches!(pkg::header(&MemoryIo::new(data), PKG_SIZE as u64), Err(Error::Unsupported(_))));

    let mut data = fih();
    data[..4].copy_from_slice(b"\x7fCNT");
    assert!(matches!(pkg::header(&MemoryIo::new(data), PKG_SIZE as u64), Err(Error::NotPs5(_))));
}

#[test]
fn signed_retail_header_is_not_treated_as_debug_fpkg() {
    let mut data = fih();
    data[5] = 1;
    assert!(matches!(pkg::header(&MemoryIo::new(data), PKG_SIZE as u64), Err(Error::Unsupported(_))));
}

#[test]
fn malformed_fih_offsets_are_rejected() {
    let mut data = fih();
    le64_at(&mut data, 0x18, 0);
    assert!(matches!(pkg::header(&MemoryIo::new(data), PKG_SIZE as u64), Err(Error::Corrupt(_))));

    let mut data = fih();
    le64_at(&mut data, 0x10, pkg::BLOCK + 1);
    assert!(matches!(pkg::header(&MemoryIo::new(data), PKG_SIZE as u64), Err(Error::Corrupt(_))));

    let mut data = fih();
    le64_at(&mut data, 0x20, pkg::BLOCK * 3);
    assert!(matches!(pkg::header(&MemoryIo::new(data), PKG_SIZE as u64), Err(Error::Corrupt(_))));

    let mut data = fih();
    le64_at(&mut data, 0x58, PKG_SIZE as u64);
    assert!(matches!(pkg::header(&MemoryIo::new(data), PKG_SIZE as u64), Err(Error::Corrupt(_))));
}

#[test]
fn valid_cnt_entries_are_found_and_preserve_encryption_flag() {
    let io = MemoryIo::new(with_cnt(fih())).chunked(19);
    let container = pkg::container(&io, PKG_SIZE as u64, CNT_AT as u64).unwrap();
    assert_eq!(container.content_id, "TEST-ID");
    assert_eq!(container.entries.len(), 2);
    let icon = container.entry(0x1200).unwrap();
    assert_eq!(icon.at, (CNT_AT + 0x1100) as u64);
    assert_eq!(icon.size, 48);
    assert_eq!(icon.flags1, 0);
    assert_eq!(container.entry(0x2000).unwrap().flags1, 0x8000_0000);
    assert!(container.entry(0xdead).is_none());
}

#[test]
fn oversized_cnt_table_is_rejected_before_allocating_entries() {
    let mut data = with_cnt(fih());
    be32_at(&mut data, CNT_AT + 0x10, 4097);
    let io = MemoryIo::new(data);
    assert!(matches!(pkg::container(&io, PKG_SIZE as u64, CNT_AT as u64), Err(Error::Corrupt(_))));
}

#[test]
fn cnt_entry_pointing_beyond_package_is_rejected() {
    let mut data = with_cnt(fih());
    be32_at(&mut data, CNT_TABLE_AT + 20, u32::MAX);
    assert!(matches!(pkg::container(&MemoryIo::new(data), PKG_SIZE as u64, CNT_AT as u64), Err(Error::Corrupt(_))));
}

#[test]
fn malformed_cnt_magic_and_truncated_table_fail_safely() {
    let io = MemoryIo::new(fih());
    assert!(matches!(pkg::container(&io, PKG_SIZE as u64, CNT_AT as u64), Err(Error::Corrupt(_))));

    let mut data = with_cnt(fih());
    be32_at(&mut data, CNT_AT + 0x18, (PKG_SIZE - CNT_AT - 16) as u32);
    let io = MemoryIo::new(data);
    assert!(matches!(pkg::container(&io, PKG_SIZE as u64, CNT_AT as u64), Err(Error::Corrupt(_))));
}
