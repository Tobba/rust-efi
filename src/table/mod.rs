mod system;
mod boot_services;
mod runtime_services;

use core::ops::{Deref, DerefMut};

pub use self::system::*;
pub use self::boot_services::*;
pub use self::runtime_services::*;

#[repr(C)]
pub struct Table<T> {
	signature: u64,
	revision: u32,
	size: u32,
	crc32: u32,
	reserved: u32,

	inner: T
}

impl<T> Deref for Table<T> {
	type Target = T;

	fn deref(&self) -> &T {
		&self.inner
	}
}

impl<T> DerefMut for Table<T> {
	fn deref_mut(&mut self) -> &mut T {
		&mut self.inner
	}
}
