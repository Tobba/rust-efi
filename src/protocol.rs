use core::prelude::*;
use core::mem::{size_of, uninitialized};
use core::marker::PhantomData;
use core::ptr;
use core::slice;
use io::{Read, Seek, SeekFrom, EndOfFile};

use ::{Status, Table, Handle, Guid, Time};
use table;

pub trait Protocol {
	fn guid() -> Guid;
}

#[repr(C)]
pub struct SimpleTextInput; // TODO

impl Protocol for SimpleTextInput {
	fn guid() -> Guid {
		Guid(0x387477C2, 0x69C7, 0x11D2, 0x8E, 0x39, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B)
	}
}

#[repr(C)]
pub struct SimpleTextOutput {
	reset: efi_fn!(*const SimpleTextOutput, bool),
	output_string: efi_fn!(*const SimpleTextOutput, *const u16),
	// TODO: the rest of this
}

impl Protocol for SimpleTextOutput {
	fn guid() -> Guid {
		Guid(0x387477C1, 0x69C7, 0x11D2, 0x8E, 0x39, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B)
	}
}

impl SimpleTextOutput {
	pub fn reset(&self, extended_verification: bool) -> Status {
		(self.reset)(self as *const SimpleTextOutput, extended_verification)
	}

	pub fn print(&self, string: &str) -> Status {
		let mut buffer = [0; 136];
		let mut buffer_cursor = 0;
		for c in string.chars() {
			buffer_cursor += c.encode_utf16(&mut buffer[buffer_cursor..]).unwrap();
			if buffer_cursor >= 128 {
				buffer[buffer_cursor] = 0;
				match (self.output_string)(self as *const SimpleTextOutput, buffer.as_ptr()) {
					::Status::Success => { },
					error => return error
				}
			}
		}

		if buffer_cursor > 0 {
			buffer[buffer_cursor] = 0;
			(self.output_string)(self as *const SimpleTextOutput, buffer.as_ptr())
		} else {
			::Status::Success
		}
	}
}

#[repr(C)]
pub struct LoadedImage<'a> {
	revision: u32,
	parent_handle: Handle,
	system_table: *const Table<table::System<'a>>,

	device_handle: Handle,
	device_path: *const (),
	reserved: *const (),

	load_options_size: u32,
	load_options: *const (),

	// TODO: the rest of this
}

impl<'a> Protocol for LoadedImage<'a> {
	fn guid() -> Guid {
		Guid(0x5B1B31A1, 0x9562, 0x11D2, 0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B)
	}
}

impl<'a> LoadedImage<'a> {
	pub fn get_device(&self) -> Handle {
		self.device_handle
	}
}

#[repr(C)]
pub struct SimpleFileSystem {
	revision: u64,
	open: efi_fn!(*const SimpleFileSystem, *mut *const FileProtocol)
}

impl Protocol for SimpleFileSystem {
	fn guid() -> Guid {
		Guid(0x0964E5B22, 0x6459, 0x11D2, 0x8E, 0x39, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B)
	}
}

impl SimpleFileSystem {
	pub fn open(&self) -> Option<Directory> {
		let mut file: *const FileProtocol = ptr::null();
		if (self.open)(self as *const SimpleFileSystem, &mut file as *mut *const FileProtocol) != ::Status::Success {
			return None;
		}
		Some(Directory {
			protocol: file
		})
	}
}

#[repr(C)]
struct FileProtocol { // not even a proper protocol, doesnt have a GUID (afaik), just holds all the methods for us
	revision: u64,
	open: efi_fn!(*const FileProtocol, *mut *const FileProtocol, *const u16, u64, u64),
	close: efi_fn!(*const FileProtocol),
	delete: efi_fn!(*const FileProtocol),
	read: efi_fn!(*const FileProtocol, *mut usize, *mut u8),
	write: efi_fn!(*const FileProtocol, *mut usize, *mut u8),
	get_position: efi_fn!(*const FileProtocol, *mut u64),
	set_position: efi_fn!(*const FileProtocol, u64),
	get_info: efi_fn!(*const FileProtocol, *const Guid, *mut usize, *mut FileInfo),
	// this is incomplete
}

static FILE_INFO_GUID: Guid = Guid(0x09576E92, 0x6D3F, 0x11D2, 0x8E, 0x39, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B);
struct FileInfo {
	size: u64,
	file_size: u64,
	physical_size: u64,

	created: Time,
	last_access: Time,
	last_modified: Time,

	attributes: u64,
	name: [u16; 128]
}

pub enum OpenResult {
	File(File),
	Directory(Directory),
	None
}

pub struct Directory {
	protocol: *const FileProtocol
}

impl Directory {
	pub fn open(&self, path: &str) -> OpenResult {
		let mut buffer = [0; 128];
		let mut buffer_cursor = 0;
		for c in path.chars() {
			buffer_cursor += c.encode_utf16(&mut buffer[buffer_cursor..]).unwrap();
		}
		buffer[buffer_cursor] = 0;
		let mut file_protocol = 0 as *const FileProtocol;
		if (unsafe { &*self.protocol }.open)(self.protocol, &mut file_protocol as *mut *const FileProtocol, buffer.as_ptr(), 1, 0) != ::Status::Success {
			return OpenResult::None;
		}
		let mut info: FileInfo = unsafe { uninitialized() };
		let mut size = size_of::<FileInfo>();
		if (unsafe { &*file_protocol }.get_info)(file_protocol, &FILE_INFO_GUID as *const Guid, &mut size as *mut usize, &mut info as *mut FileInfo) != ::Status::Success {
			panic!("could not read file info")
		}
		if info.attributes & 0x10 > 0 {
			OpenResult::Directory(Directory {
				protocol: file_protocol
			})
		} else {
			OpenResult::File(File {
				protocol: file_protocol
			})
		}
	}

	pub fn files<'a>(&'a mut self) -> DirectoryFiles<'a> {
		DirectoryFiles {
			protocol: self.protocol,
			marker: PhantomData
		}
	}
}

pub struct DirectoryFiles<'a> {
	protocol: *const FileProtocol,
	marker: PhantomData<&'a Directory>
}

impl<'a> Iterator for DirectoryFiles<'a> {
	type Item = ();

	fn next(&mut self) -> Option<()> {
		None
	}
}

pub struct File {
	protocol: *const FileProtocol
}

impl File {
	pub fn size(&self) -> u64 {
		let mut info: FileInfo = unsafe { uninitialized() };
		let mut size = size_of::<FileInfo>();
		if (unsafe { &*self.protocol }.get_info)(self.protocol, &FILE_INFO_GUID as *const Guid, &mut size as *mut usize, &mut info as *mut FileInfo) != ::Status::Success {
			panic!("could not read file info")
		}
		info.file_size
	}
}

impl Read for File {
	type Err = EndOfFile;

	fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndOfFile> {
		let mut length = buf.len();
		if (unsafe { &*self.protocol }.read)(self.protocol, &mut length as *mut usize, buf.as_mut_ptr()) != ::Status::Success {
			return Err(EndOfFile); // TODO: handle this correctly
		}
		if length == 0 {
			return Err(EndOfFile); // EOF
		}
		Ok(length)
	}
}

impl Seek for File {
	type Err = (); // FIXME

	fn tell(&mut self) -> Result<u64, ()> {
		let mut pos = 0;
		if (unsafe { &*self.protocol }.get_position)(self.protocol, &mut pos as *mut u64) != ::Status::Success {
			return Err(()); // TODO: handle this correctly
		}
		Ok(pos)
	}

	fn seek(&mut self, from: SeekFrom) -> Result<u64, ()> {
		let pos = match from {
			SeekFrom::Start(offset) => offset,
			SeekFrom::End(offset) => (self.size() as i64 + offset) as u64,
			SeekFrom::Current(offset) => (self.tell().unwrap() as i64 + offset) as u64
		};
		if (unsafe { &*self.protocol }.set_position)(self.protocol, pos) != ::Status::Success {
			return Err(()); // TODO: handle this correctly
		}
		Ok(pos)
	}
}

#[repr(C)]
#[derive(Debug)]
pub struct ModeInfo {
	version: u32,
	pub x_res: u32,
	pub y_res: u32,
	pub pixel_format: u32,
	pub bitmask: (u32, u32, u32, u32),
	pub stride: u32
}

#[repr(C)]
struct GraphicsMode {
	max_mode: u32,
	mode: u32,
	mode_info: *const ModeInfo,
	info_size: usize,
	framebuffer_base: u64,
	framebuffer_size: usize
}

#[repr(usize)]
enum BlitMode {
	Fill = 0,
	VideoToBuffer = 1,
	BufferToVideo = 2,
	VideoToVideo = 3,
}

#[repr(C)]
pub struct GraphicsOutput {
	query_mode: efi_fn!(*const GraphicsOutput, u32, *const usize, *const *const ModeInfo),
	set_mode: efi_fn!(*const GraphicsOutput, u32),
	blit: efi_fn!(*const GraphicsOutput, *mut u32, BlitMode, usize, usize, usize, usize, usize, usize, usize),
	mode: *const GraphicsMode
}

impl Protocol for GraphicsOutput {
	fn guid() -> Guid {
		Guid(0x9042A9DE, 0x23DC, 0x4A38, 0x96, 0xFB, 0x7A, 0xDE, 0xD0, 0x80, 0x51, 0x6A)
	}
}

impl GraphicsOutput {
	pub fn query_mode(&self, mode: u32) -> ModeInfo {
		let size = size_of::<ModeInfo>();
		let info_ptr = 0 as *const ModeInfo;
		unsafe {
			(self.query_mode)(self, mode, &size, &info_ptr); // TODO: handle errors
			ptr::read(info_ptr)
		}
	}

	pub fn get_mode_count(&self) -> u32 {
		unsafe {
			(&*self.mode).max_mode
		}
	}

	pub fn set_mode(&self, mode: u32) {
		(self.set_mode)(&*self, mode);
	}

	pub fn get_framebuffer(&self) -> Option<&mut [u8]> { // FIXME: this lets you have multiple mutable references into the framebuffer
		let mode = unsafe { &*self.mode };
		if mode.framebuffer_base == 0 {
			return None;
		}
		Some(unsafe { slice::from_raw_parts_mut(mode.framebuffer_base as *mut u8, mode.framebuffer_size) })
	}

	pub fn fill(&self, color: u32, x: usize, y: usize, width: usize, height: usize) {
		(self.blit)(self, &color as *const u32 as *mut u32, BlitMode::Fill, 0, 0, x, y, width, height, 0);
	}
}
