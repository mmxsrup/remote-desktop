extern crate repng;

use std::net::TcpStream;
use std::io::Read;
use std::fs::File;

fn recv_images(mut stream: TcpStream) {
	print!("recv_images");
	let w = 1920;
	let h = 1080;

	let mut buffer = Vec::with_capacity(w * h * 4);

	loop {
		match stream.read_to_end(&mut buffer) {
			Ok(size) => {
				println!("received {}", size);
				// println!("{:?}", buffer);

				// Save the image.
				repng::encode(
				File::create("buffer.png").unwrap(),
					w as u32,
					h as u32,
					&buffer,
				).unwrap();

			}
			Err(e) => {
				println!("Error: {}", e);
			}
		}
		break;
	}
}

fn main() {

	match TcpStream::connect("127.0.0.1:8888") {
		Ok(stream) => {
			println!("Successfully connected to server in port 8888");
			// thread::spawn(move || {
				recv_images(stream);
			// });
		}
		Err(e) => {
			// Connection failed
			println!("Error: {}", e);
		}
	}
}
