use core::prelude::*;
use core::fmt;

use stdio;

#[lang="panic_fmt"]
extern fn panic_fmt(msg: fmt::Arguments, file: &'static str, line: u32) -> ! {
	println!("Panic at {}:{}: ", file, line); // TODO: print to stderr
	stdio::println(msg);

	// TODO: exit gracefully
	loop { }
}

#[lang="stack_exhausted"]
extern fn stack_exhausted() {
	loop { }
}

#[lang="eh_personality"]
extern fn eh_personality() {
	loop { }
}
