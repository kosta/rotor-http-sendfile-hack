extern crate libc;
extern crate rotor;
extern crate rotor_http;

use std::env::args;
use std::env;
use std::ffi::CString;
use libc::{c_int, size_t};

#[cfg(target_os="macos")]
use std::boxed::Box;

#[cfg(target_os="macos")]
use std::ops::DerefMut;

use std::thread;
use std::time::Duration;

use rotor::{Scope, Time};
use rotor_http::server::{Fsm, RecvMode, Server, Head, Response};
use rotor::mio::tcp::TcpListener;

struct Context {
	base_path: String
}

#[derive(Debug, Clone)]
enum ServerState {
    Initial,
    ServeFile {
    	path: String
	}
}

fn send_string(res: &mut Response, data: &[u8]) {
    res.status(200, "OK");
    res.add_length(data.len() as u64).unwrap();
    res.done_headers().unwrap();
    res.write_body(data);
    res.done();
}

impl Server for ServerState {
    type Seed = ();
    type Context = Context;
    fn headers_received(_seed: (), 
    	head: Head, 
    	_res: &mut Response,
        scope: &mut Scope<Context>)
        -> Option<(Self, RecvMode, Time)>
    {
        Some((
        	ServerState::ServeFile{path: head.path.into()}, 
        	RecvMode::Buffered(8 * 1024), 
        	scope.now() + Duration::new(10, 0)
    	))
    }

    fn request_received(self, 
    	_data: &[u8], 
    	res: &mut Response,
        scope: &mut Scope<Context>)
        -> Option<Self>
    {
        match self {
            ServerState::Initial => {
                return None
            }
            ServerState::ServeFile { ref path } => {
                send_string(res, path.as_bytes());
            }
        }
        Some(self)
    }

    fn request_chunk(self, _chunk: &[u8], _response: &mut Response,
        _scope: &mut Scope<Context>)
        -> Option<Self>
    {
        None
    }

    /// End of request body, only for Progressive requests
    fn request_end(self, _response: &mut Response, _scope: &mut Scope<Context>)
        -> Option<Self>
    {
        None
    }

    fn timeout(self, _response: &mut Response, _scope: &mut Scope<Context>)
        -> Option<(Self, Time)>
    {
        None
    }

    fn wakeup(self, _response: &mut Response, _scope: &mut Scope<Context>)
        -> Option<Self>
    {
        unimplemented!();
    }
}

fn main() {
	let mut args = args();
	let num_args = args.len();
	if num_args != 2 {
		println!("Expected 2 arguments but got {}", num_args);
		std::process::exit(1);
	}
	let base_path = args.nth(1).unwrap();

    println!("Starting http server on http://127.0.0.1:3000/");
    let lst = TcpListener::bind(&"127.0.0.1:3000".parse().unwrap()).unwrap();
    let threads = env::var("THREADS").unwrap_or("2".to_string()).parse().unwrap();
    let mut children = Vec::new();
    for _ in 0..threads {
        let listener = lst.try_clone().unwrap();
        let base_path_clone = base_path.clone();
        children.push(thread::spawn(move || {
            let event_loop = rotor::Loop::new(
                &rotor::Config::new()).unwrap();
            let mut loop_inst = event_loop.instantiate(Context {
            	base_path: base_path_clone
            });
            loop_inst.add_machine_with(|scope| {
                Fsm::<ServerState, _>::new(listener, (), scope)
            }).unwrap();
            loop_inst.run().unwrap();
        }));
    }
    for child in children {
        child.join().unwrap();
    }
}


/*
fn main() {
	println!("copying using sendfile() from '{}' to '{}'", 
		src_path.replace("\"", "\\\""), 
		dst_path.replace("\"", "\\\""));

	let src = open(&src_path, libc::O_RDONLY);
	let count = std::fs::metadata(src_path).unwrap().len() as size_t;
	let dst = open(&dst_path, libc::O_WRONLY | libc::O_EXCL | libc::O_CREAT);
	sendfile(src, dst, count);
}
*/

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
