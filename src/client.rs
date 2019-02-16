extern crate repng;
extern crate gtk;
extern crate gdk_pixbuf;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;


use std::net::TcpStream;
use std::io::Read;
use relm::{Relm, Update, Widget, interval};
use gtk::prelude::*;
use gtk::{Window, Inhibit, WindowType, Image, ImageExt};
use gdk_pixbuf::{Pixbuf, Colorspace, InterpType, PixbufExt};
use gtk::Orientation::Vertical;
use std::env;
use std::process;

mod command;
use command::Command;


fn connect(addr: &str) -> TcpStream {
	println!("connect");
	match TcpStream::connect(addr) {
		Ok(stream) => {
			println!("Successfully connected to server addr:{}", addr);
			stream
		},
		Err(e) => {
			panic!("Error: {}", e);
		}
	}
}

fn recv_images(model: &mut Model) -> Vec<u8> {
	println!("recv_images");

	// let mut buffer: Vec<u8> = Vec::with_capacity(w * h * 4);
	let bufsize = model.width * model.height * 4;
	let mut buffer = vec![0u8; bufsize as usize];

	match model.stream.read_exact(&mut buffer) {
		Ok(_) => {
			println!("Read successfully");
		}
		Err(e) => {
			println!("Error: {}", e);
		}
	};
	buffer
}

struct Model {
	stream: TcpStream,
	width: i32,
	height: i32,
	ratio: f32,
}

#[derive(Msg)]
enum Msg {
	Draw,
	Mouse((f64, f64), u32),
	Key((u32)),
	Quit,
}

struct Win {
	image: Image,
	model: Model,
	window: Window,
}

impl Update for Win {
	// Specify the model used for this widget.
	type Model = Model;
	// Specify the model parameter used to init the model.
	type ModelParam = TcpStream;
	// Specify the type of the messages sent to the update function.
	type Msg = Msg;

	// Return the initial model.
	fn model(_: &Relm<Self>, stream: TcpStream) -> Model {
		Model {
			stream: stream,
			width: 1920,
			height: 1080,
			ratio: 0.8,
		}
	}

	fn subscriptions(&mut self, relm: &Relm<Self>) {
		interval(relm.stream(), 1000, || Msg::Draw);
	}

	// Widgets may also be updated in this function.
	fn update(&mut self, event: Msg) {
		match event {
			Msg::Draw => {
				println!("Draw");
				let buffer = recv_images(&mut self.model);

				let pixbuf = Pixbuf::new_from_vec(
					buffer,
					Colorspace::Rgb,
					true,
					8,
					self.model.width,
					self.model.height,
					self.model.width * 4);
				let pixbuf_small = pixbuf.scale_simple(
					(self.model.width as f32 * self.model.ratio) as i32,
					(self.model.height as f32 * self.model.ratio) as i32,
					InterpType::Bilinear);
				self.image.set_from_pixbuf(&pixbuf_small);
			},
			Msg::Mouse(pos, button) => {
				println!("Mouse {:?} {:?}", pos, button);
				let mut command = Command::new();
				command.set_mouse(
					((pos.0 as f32 / self.model.ratio) as i32,
					(pos.1 as f32 / self.model.ratio) as i32),
					button as i32
				);
				command.send(&mut self.model.stream);
			},
			Msg::Key(keyval) => {
				println!("Key {:?}", keyval);
				let mut command = Command::new();
				command.set_key(keyval as u8);
				command.send(&mut self.model.stream);
			},
			Msg::Quit => gtk::main_quit(),
		}
	}
}

impl Widget for Win {
	// Specify the type of the root widget.
	type Root = Window;

	// Return the root widget.
	fn root(&self) -> Self::Root {
		self.window.clone()
	}

	// Create the widgets.
	fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
		let vbox = gtk::Box::new(Vertical, 0);

		let image = Image::new();
		vbox.add(&image);

		let window = Window::new(WindowType::Toplevel);
		window.add(&vbox);

		// Event when mouse button pressed
		connect!(relm, window, connect_button_press_event(_, event),
			return(Some(Msg::Mouse(event.get_position(), event.get_button())), Inhibit(false)));
		// Event when pressing keyboard
		connect!(relm, window, connect_key_press_event(_, event),
			return(Some(Msg::Key(event.get_keyval())), Inhibit(false)));
		connect!(relm, window, connect_delete_event(_, _), return (Some(Msg::Quit), Inhibit(false)));

		window.show_all();

		let mut win = Win {
			image: image,
			model,
			window: window,
		};

		win.update(Msg::Draw);
		win
	}
}

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() != 2 {
		println!("Usage: cargo run --bin client server_addr:port");
		process::exit(1);
	}

	let addr = &args[1];
	let stream = connect(&addr);
	Win::run(stream).expect("Win::run failed");
}
