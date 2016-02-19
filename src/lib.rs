#![no_std]
#![feature(lang_items, no_std, type_macros)]
#![feature(core, collections, libc, unicode, core_prelude)]

extern crate libc;
#[macro_use]
extern crate collections;
extern crate coreio as io;

use core::ops::{Deref, DerefMut};
use core::fmt;

#[cfg(target_pointer_width="32")]
macro_rules! efi_fn {
	($($typ:ty),*) => (extern "system" fn($($typ),*) -> $crate::Status)
}

#[cfg(target_pointer_width="64")]
macro_rules! efi_fn {
	($($typ:ty),*) => (extern "win64" fn($($typ),*) -> $crate::Status)
}

#[macro_use]
pub mod stdio;
#[macro_use]
pub mod entry;
pub mod table;
pub mod protocol;
pub mod panic;
pub mod mem;

pub use table::Table;

mod std { pub use core::*; }

#[repr(usize)]
#[derive(PartialEq, Eq)]
pub enum Status {
	Success = 0,
	Aaaa = 1,
}

pub static mut system_table: *const Table<table::System<'static>> = 0 as *const Table<table::System<'static>>;
pub static mut boot_services: *const Table<table::BootServices> = 0 as *const Table<table::BootServices>;
pub static mut runtime_services: *const Table<table::RuntimeServices> = 0 as *const Table<table::RuntimeServices>;
pub static mut current_image: Handle = Handle { _ptr: 0 as *const () };

pub fn get_system_table() -> &'static Table<table::System<'static>> {
	unsafe {
		&*system_table
	}
}

pub fn get_boot_services() -> &'static Table<table::BootServices> {
	unsafe {
		&*boot_services
	}
}

pub fn get_runtime_services() -> &'static Table<table::RuntimeServices> {
	unsafe {
		&*runtime_services
	}
}

pub fn get_current_image() -> Handle {
	unsafe {
		current_image
	}
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Guid(pub u32, pub u16, pub u16, pub u8, pub u8, pub u8, pub u8, pub u8, pub u8, pub u8, pub u8);

impl fmt::Display for Guid {
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		let Guid(a, b, c, d, e, f, g, h, i, j, k) = *self;
		formatter.write_fmt(format_args!("{:X}-{:X}-{:X}-{:X}-{:X}-{:X}-{:X}-{:X}-{:X}-{:X}-{:X}", a, b, c, d, e, f, g, h, i, j, k))
	}
}

impl fmt::Debug for Guid {
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, formatter)
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug)] // TODO: should this really be Copy?
pub struct Handle {
	_ptr: *const ()
}

impl Handle {
	pub fn get_protocol<T: protocol::Protocol>(&self) -> Option<&T> {
		let mut ptr: *const T = 0 as *const T;
		unsafe {
			get_boot_services().handle_protocol(*self, <T as protocol::Protocol>::guid(), &mut ptr as *mut *const T as *mut *const ()); // TODO: check the error
		}
		if ptr.is_null() {
			None
		} else {
			unsafe {
				Some(&*ptr)
			}
		}
	}
}

#[repr(C)]
pub struct Time { // apparently we're just too cool to use a normal goddamn UNIX timestamp, or any other format remotely sensible to store time internally in
	year: u16,
	month: u8,
	day: u8,
	hour: u8,
	minute: u8,
	second: u8,
	pad1: u8,
	nanosecond: u32,
	time_zone: i16,
	daylight: u8,
	pad2: u8
}
