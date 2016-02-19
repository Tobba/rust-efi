use table;
use Table;
use Handle;

#[macro_export]
macro_rules! efi_main {($name:ident) => {
	#[no_mangle]
	pub extern "C" fn rust_efi_main() -> $crate::Status {
		$name()
	}
}}

extern "C" {
	fn rust_efi_main() -> ::Status;
}

#[cfg(target_pointer_width="32")]
#[no_mangle]
pub extern "system" fn efi_main(image: Handle, system_table: &'static Table<table::System<'static>>) -> ::Status {
	unsafe {
		::system_table = system_table as *const Table<table::System>;
		::boot_services = system_table.get_boot_services();
		::runtime_services = system_table.get_runtime_services();
		::current_image = image;
		rust_efi_main()
	}
}

#[cfg(target_pointer_width="64")]
#[no_mangle]
pub extern "win64" fn efi_main(image: Handle, system_table: &'static Table<table::System<'static>>) -> ::Status {
	unsafe {
		::system_table = system_table as *const Table<table::System>;
		::boot_services = system_table.get_boot_services();
		::runtime_services = system_table.get_runtime_services();
		::current_image = image;
		rust_efi_main()
	}
}
