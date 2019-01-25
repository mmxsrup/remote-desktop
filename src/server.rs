extern crate scrap;

use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;
use std::net::{TcpListener, TcpStream};
use std::io::Write;

fn send_images(mut stream: TcpStream) {
	let one_second = Duration::new(1, 0);
	let one_frame = one_second / 60;

	let display = Display::primary().expect("Couldn't find primary display.");
	let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
	let (w, h) = (capturer.width(), capturer.height());
	println!("{} {}", w, h);

	loop {
		// Wait until there's a frame.

		// buffer is ARGB image
		let buffer = match capturer.frame() {
			Ok(buffer) => buffer,
			Err(error) => {
				if error.kind() == WouldBlock {
					// Keep spinning.
					thread::sleep(one_frame);
					continue;
				} else {
					panic!("Error: {}", error);
				}
			}
		};

        // Flip the ARGB image into a BGRA image.
		let mut bitflipped = Vec::with_capacity(w * h * 4);
		let stride = buffer.len() / h;

		for y in 0..h {
			for x in 0..w {
				let i = stride * y + 4 * x;
				bitflipped.extend_from_slice(&[
					buffer[i + 2],
					buffer[i + 1],
					buffer[i],
					255,
				]);
			}
		}

		// println!("{:?}", bitflipped);
		stream.write(&bitflipped).unwrap();
		break;
	}
}


fn main() {

	let listener = TcpListener::bind("127.0.0.1:8888").unwrap();
	println!("Server listening on port 8888");

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				// Connection succeeded
				println!("New connection: {}", stream.peer_addr().unwrap());
				// thread::spawn(move || {
					send_images(stream);
				// });
			}
			Err(e) => {
				// Connection failed
				println!("Error: {}", e);
			}
		}
	}

	// close the socket server
	drop(listener);
}
