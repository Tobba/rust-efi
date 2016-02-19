use core::prelude::*;
use core::ptr;
use libc;

use table::{MemoryType, AllocType};

#[no_mangle]
pub unsafe extern fn malloc(size: libc::size_t) -> *mut libc::c_void {
	::get_boot_services().alloc(MemoryType::LoaderData, size as usize).expect("out of memory") as *mut libc::c_void
}

// TODO: care about the alignment
#[no_mangle]
pub unsafe extern fn posix_memalign(ptr: *mut *mut libc::c_void, align: libc::size_t, size: libc::size_t) -> libc::c_int {
	*ptr = malloc(size);
	0
}

#[no_mangle]
pub unsafe extern fn realloc(old: *mut libc::c_void, size: libc::size_t) -> *mut libc::c_void {
	let ptr = malloc(size);
	// this is a hack that abuses that copying outside of where we should wont generate exceptions
	// this is still a horrible idea since we could read into MMIO space
	ptr::copy(old as *const u8, ptr as *mut u8, size as usize);
	ptr
}

#[no_mangle]
pub unsafe extern fn free(ptr: *mut libc::c_void) {
	::get_boot_services().free(ptr as *mut ());
}

pub struct PageAlloc {
	ptr: *mut (),
	count: usize
}

impl PageAlloc {
	pub fn get_ptr(&self) -> *mut () {
		self.ptr
	}

	pub fn get_size(&self) -> usize {
		self.count * 4096
	}
}

impl Drop for PageAlloc {
	fn drop(&mut self) {
		unsafe {
			::get_boot_services().free_pages(self.ptr, self.count);
		}
	}
}

pub enum AllocAt {
	Anywhere,
	Below(*const ()),
	At(*mut ())
}

pub fn alloc_pages(at: AllocAt, memory_type: MemoryType, count: usize) -> Option<PageAlloc> {
	let (alloc_type, address) = match at {
		AllocAt::Anywhere => (AllocType::AnyPages, ptr::null_mut()),
		AllocAt::Below(address) => (AllocType::MaxAddress, address as *mut ()),
		AllocAt::At(ptr) => (AllocType::Address, ptr)
	};
	let ptr = match unsafe { ::get_boot_services().alloc_pages(alloc_type, memory_type, count, address) } {
		Some(ptr) => ptr,
		None => return None
	};
	Some(PageAlloc {
		ptr: ptr,
		count: count
	})
}