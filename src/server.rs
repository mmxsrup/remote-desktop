extern crate scrap;
extern crate enigo;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{BufWriter, BufReader};
use std::io::prelude::*;
use std::env;
use std::process;

mod image;
use image::Image;
mod command;
use command::Command;


fn send_images(writer: &mut BufWriter<TcpStream>, fps: u32) {

	let mut image = Image::new();
	image.set_one_frame(fps);

	loop {
		image.update();
		image.send(writer);

		thread::sleep(image.get_one_frame());
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