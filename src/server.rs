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
use std::io::prelude::*;
use std::env;
use std::process;

mod command;
use command::Command;


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

	loop {
		let mut buf = String::new();
		reader.read_line(&mut buf).unwrap();
		println!("recv buf {:?}", buf);

		let command = Command::recv(buf);
		command.purse();
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
