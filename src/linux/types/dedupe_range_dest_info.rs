use std::os::unix::prelude::RawFd;

use crate::linux::imports::*;

#[ derive (Debug, Eq, PartialEq) ]
pub struct DedupeRangeDestInfo {
	pub dest_fd: RawFd,
	pub dest_offset: u64,
	pub bytes_deduped: u64,
	pub status: DedupeRangeStatus,
}

// ex: noet ts=4 filetype=rust
