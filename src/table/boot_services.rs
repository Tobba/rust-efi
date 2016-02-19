use core::ptr;
use core::mem::{size_of, uninitialized};
use collections::Vec;
use ::{Status, Guid, Handle};

#[repr(usize)]
pub enum Tpl {
	Application = 4,
	Callback = 8,
	Notify = 16,
	HighLevel = 31
}

#[repr(usize)]
pub enum TimerType {
	Cancel = 0,
	Periodic = 1,
	Relative = 2
}

#[repr(C, packed)]
pub struct BootServices {
	raise_tpl: efi_fn!(Tpl), // FIXME: this returns a Tpl, not a Status
	restore_tpl: efi_fn!(Tpl), // FIXME: doesn't return anything

	allocate_pages: efi_fn!(AllocType, MemoryType, usize, *mut u64),
	free_pages: efi_fn!(u64, usize),
	get_memory_map: efi_fn!(*mut usize, *mut (), *mut usize, *mut usize, *mut u32),
	allocate_pool: efi_fn!(MemoryType, usize, *mut *mut ()),
	free_pool: efi_fn!(*mut ()),

	create_event: efi_fn!(usize, Tpl, efi_fn!(), usize, *mut *const ()), // TODO: first arg should be an EventType bitfield
	set_timer: efi_fn!(*const (), TimerType, u64),
	wait_for_event: efi_fn!(usize, *const *const (), *mut usize),
	signal_event: efi_fn!(*const ()),
	close_event: efi_fn!(*const ()),
	check_event: efi_fn!(*const ()),

	install_protocol_interface: *const (),
	reinstall_protocol_interface: *const (),
	uninstall_protocol_interface: *const (),
	handle_protocol: efi_fn!(Handle, &Guid, *mut *mut ()),
	reserved: *const (),
	register_protocol_notify: *const (),
	locate_handle: efi_fn!(SearchType, *const Guid, *const (), *mut usize, *mut Handle),
	locate_device_path: *const (),
	install_configuration_table: *const (),

	load_image: *const (),
	start_image: *const (),
	exit: *const (),
	unload_image: *const (),
	exit_boot_services: efi_fn!(Handle, usize),

	// this is incomplete
}

#[repr(u32)]
enum SearchType {
	AllHandles = 0,
	ByRegisterNotify = 1,
	ByProtocol = 2,
}

impl BootServices {
	pub unsafe fn handle_protocol(&self, handle: Handle, guid: Guid, ptr: *mut *mut ()) -> Status {
		(self.handle_protocol)(handle, &guid, ptr)
	}

	pub fn alloc(&self, typ: MemoryType, size: usize) -> Option<*mut ()> {
		let mut ptr = ptr::null_mut();
		if (self.allocate_pool)(typ, size, &mut ptr) != ::Status::Success {
			return None;
		}
		Some(ptr)
	}

	pub unsafe fn free(&self, ptr: *mut ()) {
		(self.free_pool)(ptr);
	}

	pub unsafe fn alloc_pages(&self, alloc_type: AllocType, memory_type: MemoryType, count: usize, address: *mut ()) -> Option<*mut ()> {
		let mut ptr = address as u64;
		if (self.allocate_pages)(alloc_type, memory_type, count, &mut ptr) != ::Status::Success {
			return None;
		}
		Some(ptr as *mut ())
	}

	pub unsafe fn free_pages(&self, address: *mut (), count: usize) {
		(self.free_pages)(address as u64, count);
	}

	pub fn handles_by_protocol(&self, guid: &Guid) -> Vec<Handle> {
		unsafe {
			let mut results = vec![uninitialized(); 32];
			loop {
				let mut buffer_size = results.len() * size_of::<Handle>();
				if (self.locate_handle)(SearchType::ByProtocol, &*guid, 0 as *const (), &mut buffer_size, results.as_mut_ptr()) == ::Status::Success {
					results.set_len(buffer_size / size_of::<Handle>());
					return results;
				}
				let len = results.len();
				results.extend((0..len).map(|_| uninitialized::<Handle>()));
				// TODO: handle other errors properly
			}
		}
	}

	pub fn memory_map(&self) -> (MemoryMap, usize) {
		let mut size = 0;
		let mut key = 0;
		let mut descriptor_size = 0usize;
		let mut descriptor_version = 0u32;
		(self.get_memory_map)(&mut size, ptr::null_mut(), &mut key, &mut descriptor_size, &mut descriptor_version);
		size += descriptor_size; // the allocation may end up inserting another entry
		let mem = self.alloc(MemoryType::LoaderData, size).expect("out of memory");
		if (self.get_memory_map)(&mut size as *mut usize, mem, &mut key as *mut usize, &mut descriptor_size as *mut usize, &mut descriptor_version) != ::Status::Success {
			panic!("failed to fetch memory map")
		}
		(MemoryMap {
			mem: mem,
			size: size,
			descriptor_size: descriptor_size
		}, key)
	}

	pub unsafe fn exit_boot_services(&self, image: Handle, key: usize) -> Status {
		(self.exit_boot_services)(image, key)
	}
}

#[derive(Clone, Copy)]
#[repr(C)]
pub enum AllocType {
	AnyPages,
	MaxAddress,
	Address,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum MemoryType {
	Reserved,
	LoaderCode,
	LoaderData,
	BootServicesCode,
	BootServicesData,
	RuntimeServicesCode,
	RuntimeServicesData,
	Conventional,
	Unusable,
	AcpiReclaimable,
	AcpiMemoryNvs,
	MemoryMappedIo,
	MemoryMappedIoPortSpace,
	PalCode
}

impl MemoryType {
	pub fn from_code(code: u32) -> Option<MemoryType> {
		Some(match code {
			0 => MemoryType::Reserved,
			1 => MemoryType::LoaderCode,
			2 => MemoryType::LoaderData,
			3 => MemoryType::BootServicesCode,
			4 => MemoryType::BootServicesData,
			5 => MemoryType::RuntimeServicesCode,
			6 => MemoryType::RuntimeServicesData,
			7 => MemoryType::Conventional,
			8 => MemoryType::Unusable,
			9 => MemoryType::AcpiReclaimable,
			10 => MemoryType::AcpiMemoryNvs,
			11 => MemoryType::MemoryMappedIo,
			12 => MemoryType::MemoryMappedIoPortSpace,
			13 => MemoryType::PalCode,
			_ => return None
		})
	}
}

#[repr(C)]
pub struct MemoryDescriptor {
	pub typ: u32,
	pub pad: u32,
	pub phys: u64,
	pub virt: u64,
	pub count: u64,
	pub attribute: u64
}

pub struct MemoryMap {
	mem: *mut (),
	size: usize,
	descriptor_size: usize
}

impl MemoryMap {
	pub fn get_descriptor(&self, index: usize) -> &MemoryDescriptor {
		if index * self.descriptor_size >= self.size {
			panic!()
		}
		unsafe {
			&*((self.mem as *const u8).offset((index * self.descriptor_size) as isize) as *const MemoryDescriptor)
		}
	}

	pub fn get_descriptor_count(&self) -> usize {
		self.size / self.descriptor_size
	}

	pub fn iter<'b>(&'b self) -> MemoryMapIterator<'b> {
		MemoryMapIterator {
			map: self,
			index: 0
		}
	}
}

impl Drop for MemoryMap {
	fn drop(&mut self) {
		// TODO: free
	}
}

pub struct MemoryMapIterator<'a> {
	map: &'a MemoryMap,
	index: usize
}

impl<'a> Iterator for MemoryMapIterator<'a> {
	type Item = (*const (), usize, MemoryType);

	fn next(&mut self) -> Option<(*const (), usize, MemoryType)> {
		if self.index >= self.map.get_descriptor_count() {
			return None;
		}
		let descriptor = self.map.get_descriptor(self.index);
		self.index += 1;
		Some((descriptor.phys as *const (), descriptor.count as usize * 4096, match MemoryType::from_code(descriptor.typ) { Some(typ) => typ, None => MemoryType::Reserved }))
	}
}
