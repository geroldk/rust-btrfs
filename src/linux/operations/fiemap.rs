use std::{fs::OpenOptions, os::unix::prelude::AsRawFd};

use crate::linux::imports::*;

// --------- high level wrapper

#[ derive (Debug, Eq, PartialEq) ]
pub struct FileExtent {
	pub logical: u64,
	pub physical: u64,
	pub length: u64,
}

pub fn get_file_extent_map_for_path <
	PathRef: AsRef <Path>
> (
	file_path: PathRef,
) -> io::Result <Vec <FileExtent>> {

	let file_descriptor = OpenOptions::new().read(true).write(false).open(file_path)?;

	get_file_extent_map (
		file_descriptor.as_raw_fd())

}

pub fn get_file_extent_map (
	file_descriptor: libc::c_int,
) -> io::Result <Vec <FileExtent>> {

	// call ioctl in loop to get all extents

	let mut extent_count = 128;
	let mut c_file_extent_map;

	loop {

		c_file_extent_map =
			(
				get_c_file_extent_map (
					file_descriptor,
					extent_count))?;

		if extent_count != 0 {

			let last_mapped_extent =
				c_file_extent_map.extents [
					c_file_extent_map.info.mapped_extents as usize - 1];

			if last_mapped_extent.flags & FIEMAP_EXTENT_LAST != 0 {
				break;
			}

			extent_count = 0;

		} else {

			extent_count =
				c_file_extent_map.info.mapped_extents + 16;

		}

	}

	// create return value

	Ok (
		c_file_extent_map.extents [
			0 .. c_file_extent_map.info.mapped_extents as usize
		].iter ().map (
			|c_file_extent|

			FileExtent {
				logical: c_file_extent.logical,
				physical: c_file_extent.physical,
				length: c_file_extent.length,
			}

		).collect ()
	)

}

// --------- low level wrapper

struct CFileExtentMapResult {
	info: IoctlFiemap,
	extents: Vec <IoctlFiemapExtent>,
}

fn get_c_file_extent_map (
	file_descriptor: libc::c_int,
	extent_count: u32,
) -> io::Result <CFileExtentMapResult> {

	// get file size

	let mut c_stat: libc::stat =
		unsafe { mem::zeroed () };

	let stat_result =
		unsafe {
			libc::fstat (
				file_descriptor,
				& mut c_stat as * mut libc::stat)
		};

	if stat_result != 0 {

		return Err (
			io::Error::last_os_error());

	};

	// allocate buffer

	let c_fiemap_buffer_size =
		mem::size_of::<IoctlFiemap> ()
		+ extent_count as usize
			* mem::size_of::<IoctlFiemapExtent> ();

	let mut c_fiemap_buffer: Vec <u8> =
		Vec::from_iter (
			iter::repeat (0u8).take (
				c_fiemap_buffer_size));

	let (c_fiemap_info_buffer, c_fiemap_extents_buffer) =
		c_fiemap_buffer.split_at_mut (
			mem::size_of::<IoctlFiemap> ());

	// split buffer

	let c_fiemap_info_slice: & mut [IoctlFiemap] =
		unsafe {
			slice::from_raw_parts_mut (
				mem::transmute (
					c_fiemap_info_buffer.as_mut_ptr ()),
				1)
		};

	let c_fiemap_info =
		& mut c_fiemap_info_slice [0];

	let c_fiemap_extents: & mut [IoctlFiemapExtent] =
		unsafe {
			slice::from_raw_parts_mut (
				mem::transmute (
					c_fiemap_extents_buffer.as_mut_ptr ()),
				extent_count as usize)
		};

	// get filesystem info

	c_fiemap_info.start = 0;
	c_fiemap_info.length = c_stat.st_size as u64;
	c_fiemap_info.extent_count = extent_count;
	c_fiemap_info.flags = 0; //FIEMAP_FLAG_SYNC;

	unsafe {

		ioctl_fiemap (
			file_descriptor,
			c_fiemap_info as * mut IoctlFiemap)

	}?;

	// return

	Ok (
		CFileExtentMapResult {
			info: * c_fiemap_info,
			extents: c_fiemap_extents.to_vec (),
		}
	)

}

//const FIEMAP_FLAG_SYNC: u32 = 0x00000001;

const FIEMAP_EXTENT_LAST: u32 = 0x00000001;

// ex: noet ts=4 filetype=rust
