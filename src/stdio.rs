use core::prelude::*;
use core::fmt;

use protocol;

#[macro_export]
macro_rules! println {
	($($arg:tt)*) => ($crate::stdio::println(format_args!($($arg)*)))
}

pub fn println(args: fmt::Arguments) {
	let output = ::get_system_table().get_stdout();

	struct PrintWriter<'a> {
		output: &'a protocol::SimpleTextOutput
	}
	impl
	<'a> fmt::Write for PrintWriter<'a> {
		fn write_str(&mut self, s: &str) -> fmt::Result {
			self.output.print(s);
			Ok(())
		}
	}

	{
		let mut writer = PrintWriter {
			output: output
		};
		fmt::write(&mut writer, args);
	}
	output.print("\r\n");
}
