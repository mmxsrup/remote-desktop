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
use enigo::{Enigo, KeyboardControllable, Key, MouseControllable, MouseButton};
use std::io::prelude::*;
use std::env;
use std::process;


#[derive(Serialize, Deserialize, Debug)]
struct Command {
	name: char,
	keyval: u8, // for command K(Key)
	pos: (i32, i32), // for command M(Mouse)
	button: i32, // for name M(Mouse)
}

fn send_images(writer: &mut BufWriter<TcpStream>, fps: u32) {

	let one_second = Duration::new(1, 0);
	let one_frame = one_second / fps;
	println!("frame per second: {}", fps);

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
				println!("command K {:?}", command.keyval);
				match command.keyval {
					8   => enigo.key_click(Key::Backspace), // Error
					9   => enigo.key_click(Key::Tab),
					13  => enigo.key_click(Key::Return),
					32  => enigo.key_click(Key::Space), // Error
					225 => enigo.key_click(Key::Shift),
					227 => enigo.key_click(Key::Control),
					233 => enigo.key_click(Key::Alt),
					_   => enigo.key_click(Key::Layout(command.keyval as char))
				}
		},
			'M' => {
				println!("command M");
				enigo.mouse_move_to(command.pos.0, command.pos.1);
				match command.button {
					1 => enigo.mouse_click(MouseButton::Left),
					2 => enigo.mouse_click(MouseButton::Middle),
					3 => enigo.mouse_click(MouseButton::Right),
					_ => println!("[Error] command.button value")
				}
			},
			_ => {
				println!("Non command");
			}
		}
	}
}


fn main() {

	let args: Vec<String> = env::args().collect();
	if args.len() != 2 && args.len() != 3 {
		println!("Usage: cargo run --bin server server_addr:port fps");
		process::exit(1);
	}
	let addr = &args[1];
	let fps = match args.len() {
		3 => args[2].parse().unwrap(),
		_ => 30,
	};

	let listener = TcpListener::bind(addr).unwrap();
	println!("Server listening on {}", addr);

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				// Connection succeeded
				println!("New connection: {}", stream.peer_addr().unwrap());
				let mut writer = BufWriter::new(stream.try_clone().unwrap());
				let mut reader = BufReader::new(stream);

				thread::spawn(move || {
					send_images(&mut writer, fps);
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
