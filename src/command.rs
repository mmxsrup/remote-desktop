extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::net::TcpStream;
use std::io::Write;
use enigo::{Enigo, KeyboardControllable, Key, MouseControllable, MouseButton};

#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
	name: char,
	keyval: u8, // for name K(Key)
	pos: (i32, i32), // for name M(Mouse)
	button: i32, // for name M(Mouse)
}

impl Command {

	#[allow(dead_code)]
	pub fn new() -> Command {
		Command {
			name: ' ',
			keyval: 0,
			pos: (0, 0),
			button: 0,
		}
	}

	#[allow(dead_code)]
	pub fn set_mouse(&mut self, pos: (i32, i32), button: i32) {
		self.name = 'M';
		self.keyval = 0;
		self.pos = pos;
		self.button = button;
	}

	#[allow(dead_code)]
	pub fn set_key(&mut self, keyval: u8) {
		self.name = 'K';
		self.keyval = keyval;
		self.pos = (0, 0);
		self.button = 0;
	}

	#[allow(dead_code)]
	pub fn send(&mut self, stream: &mut TcpStream) {
		let json_str = serde_json::to_string(&self).unwrap() + "\n";
		println!("Serialized Json = {}", json_str);
		stream.write(json_str.as_bytes()).unwrap();
	}

	#[allow(dead_code)]
	pub fn recv(buf: String) -> Command {
		let command: Command = serde_json::from_str(&buf).unwrap();
		command
	}

	#[allow(dead_code)]
	pub fn purse(&self) {
		let mut enigo = Enigo::new();
		match self.name {
			'K' => {
				println!("command K {:?}", self.keyval);
				match self.keyval {
					8   => enigo.key_click(Key::Backspace), // Error
					9   => enigo.key_click(Key::Tab),
					13  => enigo.key_click(Key::Return),
					32  => enigo.key_click(Key::Space), // Error
					225 => enigo.key_click(Key::Shift),
					227 => enigo.key_click(Key::Control),
					233 => enigo.key_click(Key::Alt),
					_   => enigo.key_click(Key::Layout(self.keyval as char))
				}
		},
			'M' => {
				println!("command M");
				enigo.mouse_move_to(self.pos.0, self.pos.1);
				match self.button {
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