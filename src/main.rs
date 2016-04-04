use std::env::args;
use std::ffi::CString;
use libc::{c_int, size_t};

extern crate libc;

fn main() {
	let mut args = args();
	let num_args = args.len();
	if num_args != 3 {
		println!("Expected 3 arguments but got {}", num_args);
		return;
	}
	let src_path = args.nth(1).unwrap();
	let dst_path = args.next().unwrap();

	println!("copying using sendfile() from '{}' to '{}'", 
		src_path.replace("\"", "\\\""), 
		dst_path.replace("\"", "\\\""));

	let src = open(&src_path, libc::O_RDONLY);
	let count = std::fs::metadata(src_path).unwrap().len() as size_t;
	let dst = open(&dst_path, libc::O_WRONLY | libc::O_EXCL | libc::O_CREAT);
	sendfile(src, dst, count);
}

fn open(path: &str, flags: c_int) -> c_int {
	let ret = unsafe {
		libc::open(CString::new(path).unwrap().as_ptr() as *const i8, flags)
	};
	println!("open()ing '{}' with flags {} returned fd {}", path, flags, ret);
	ret
}

fn sendfile(src: c_int, dst: c_int, count: size_t) {
	let result = unsafe {
		libc::sendfile(dst, src, std::ptr::null_mut(), count)
	};
	println!("sendfile() returned {}", result);
}
