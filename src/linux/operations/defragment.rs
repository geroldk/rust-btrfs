use std::{fs::OpenOptions, os::unix::prelude::AsRawFd};

use crate::linux::imports::*;

// --------- high level wrapper

pub fn defragment_file <
	FilePathRef: AsRef <Path>
> (
	file_path: FilePathRef,
	extent_threshold: u32,
	compression_type: CompressionType,
	flush_to_disk: bool,
) -> io::Result <()> {

	let file_path =
		file_path.as_ref ();

	let file_descriptor = OpenOptions::new().read(true).write(false).open(file_path)?;


	defragment_range (
		file_descriptor.as_raw_fd(),
		0,
		-1_i64 as u64,
		extent_threshold,
		compression_type,
		flush_to_disk,
	)

}

pub fn defragment_range (
	file_descriptor: libc::c_int,
	start: u64,
	len: u64,
	extent_threshold: u32,
	compression_type: CompressionType,
	flush_to_disk: bool,
) -> io::Result <()> {

	let defrag_range_args =
		IoctlDefragRangeArgs {
			start: start,
			len: len,
			flags: (
				if compression_type != CompressionType::None {
					DEFRAG_RANGE_COMPRESS
				} else { 0 }
			|
				if flush_to_disk {
					DEFRAG_RANGE_START_IO
				} else { 0 }
			),
			extent_thresh: extent_threshold,
			compress_type: compression_type.into (),
			unused_0: 0,
			unused_1: 0,
			unused_2: 0,
			unused_3: 0,
		};

	// call ioctl

	unsafe {

		ioctl_defrag_range (
			file_descriptor,
			& defrag_range_args as * const IoctlDefragRangeArgs,
		)

	}?;

	// return ok

	Ok (())

}

// ex: noet ts=4 filetype=rust
