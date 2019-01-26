
extern crate repng;
extern crate gtk;
extern crate gdk_pixbuf;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use std::net::TcpStream;
use std::io::Read;
use relm::{Relm, Update, Widget, interval};
use gtk::prelude::*;
use gtk::{Window, Inhibit, WindowType, Image, ImageExt, Label};
use gdk_pixbuf::{Pixbuf, Colorspace,};
use gtk::Orientation::Vertical;


fn connect() -> TcpStream {
	println!("connect");
	match TcpStream::connect("127.0.0.1:8888") {
		Ok(stream) => {
			println!("Successfully connected to server in port 8888");
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
}

#[derive(Msg)]
enum Msg {
	Draw,
	Mouse((f64, f64)),
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
	type ModelParam = ();
	// Specify the type of the messages sent to the update function.
	type Msg = Msg;

	// Return the initial model.
	fn model(_: &Relm<Self>, _: ()) -> Model {
		let stream = connect();
		Model {
			stream: stream,
			width: 1920,
			height: 1080,
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
				self.image.set_from_pixbuf(&pixbuf);
			},
			Msg::Mouse(pos) => {
				println!("Mouse {:?}", pos);
			},
			Msg::Key(keyval) => {
				println!("Key {:?}", keyval);
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

		let state_label = Label::new("Remote Desktop");
		vbox.add(&state_label);

		let image = Image::new();
		vbox.add(&image);

		let window = Window::new(WindowType::Toplevel);
		window.add(&vbox);

		// Event when mouse button pressed
		connect!(relm, window, connect_button_press_event(_, event),
			return(Some(Msg::Mouse(event.get_position())), Inhibit(false)));
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
	Win::run(()).expect("Win::run failed");
}
