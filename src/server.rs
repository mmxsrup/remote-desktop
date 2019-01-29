extern crate scrap;
extern crate enigo;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;
use std::net::{TcpListener, TcpStream};
use std::io::{Write, BufWriter, BufReader};
use enigo::{Enigo, KeyboardControllable, Key,  MouseControllable};
use std::io::prelude::*;


#[derive(Serialize, Deserialize, Debug)]
struct Command {
	name: char,
	keyval: u8, // for command K(Key)
	pos: (i32, i32), // for command M(Mouse)
}

fn send_images(writer: &mut BufWriter<TcpStream>) {

	let one_second = Duration::new(1, 0);
	let one_frame = one_second / 10;

	let display = Display::primary().expect("Couldn't find primary display.");
	let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
	let (w, h) = (capturer.width(), capturer.height());
	println!("{} {}", w, h);

	loop {
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

		writer.write(&bitflipped).unwrap();
		thread::sleep(one_frame);
	}
}

fn recv_commands(reader: &mut BufReader<TcpStream>) {

	let mut enigo = Enigo::new();

	loop {
		let mut buf = String::new();
		reader.read_line(&mut buf).unwrap();
		println!("recv buf {:?}", buf);

		let command: Command = serde_json::from_str(&buf).unwrap();
		match command.name {
			'K' => {
				println!("command K");
				enigo.key_click(Key::Layout(command.keyval as char));
			},
			'M' => {
				println!("command M");
				enigo.mouse_move_to(command.pos.0, command.pos.1)
			},
			_ => {
				println!("Non command");
			}
		}
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
				let mut writer = BufWriter::new(stream.try_clone().unwrap());
				let mut reader = BufReader::new(stream);

				thread::spawn(move || {
					send_images(&mut writer);
				});
				thread::spawn(move || {
					recv_commands(&mut reader);
				});
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
