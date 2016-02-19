use table::{BootServices, RuntimeServices};
use core::slice;
use ::{Table, Handle, Guid};
use protocol;

#[repr(C)]
pub struct System<'a> {
	firmware_vendor: *const u16,
	firmware_revision: u32,

	console_in_handle: Handle,
	console_in: &'a protocol::SimpleTextInput,
	console_out_handle: Handle,
	console_out: &'a protocol::SimpleTextOutput,
	standard_error_handle: Handle,
	standard_error: &'a protocol::SimpleTextOutput,

	runtime_services: &'a Table<RuntimeServices>,
	boot_services: &'a Table<BootServices>,

	config_count: usize,
	config_table: *const ConfigEntry
}

#[repr(C)]
pub struct ConfigEntry  {
	pub guid: Guid,
	pub ptr: *const ()
}

impl<'a> System<'a> {
	pub fn get_stdin(&self) -> &protocol::SimpleTextInput {
		&*self.console_in
	}

	pub fn get_stdout(&self) -> &protocol::SimpleTextOutput {
		&*self.console_out
	}

	pub fn get_stderr(&self) -> &protocol::SimpleTextOutput {
		&*self.standard_error
	}

	pub fn get_boot_services(&self) -> &Table<BootServices> {
		&*self.boot_services
	}

	pub fn get_runtime_services(&self) -> &Table<RuntimeServices> {
		&*self.runtime_services
	}

	pub fn get_config_table(&self) -> &[ConfigEntry] {
		unsafe {
			slice::from_raw_parts(self.config_table, self.config_count)
		}
	}
}
