use std::env::args;
use std::ffi::CString;
use libc::{c_int, size_t};

#[cfg(target_os="macos")]
use std::boxed::Box;

#[cfg(target_os="macos")]
use std::ops::DerefMut;

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

#[cfg(target_os="macos")]
struct Sendfile (Box<libc::off_t>);

#[cfg(target_os="macos")]
fn sendfile(src: c_int, dst: c_int, _: size_t) -> Sendfile {
	println!("copying fd {} to fd {}", src, dst);
	let mut sf = Sendfile( Box::new(0) );
	let result = unsafe {
		libc::sendfile(src, 
			dst, 
			0, 
			sf.0.deref_mut() as *mut libc::off_t, 
			std::ptr::null_mut(), 
			0)
	};
	println!("sendfile() returned {}", result);
	sf
}

#[cfg(target_os="linux")]
struct Sendfile;

#[cfg(target_os="linux")]
fn sendfile(src: c_int, dst: c_int, count: size_t) -> Sendfile{
	let result = unsafe {
		libc::sendfile(dst, src, std::ptr::null_mut(), count)
	};
	println!("sendfile() returned {}", result);
	Sendfile
}
