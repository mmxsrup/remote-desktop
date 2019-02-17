extern crate scrap;
extern crate enigo;
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;
use std::net::TcpStream;
use std::io::{Write, Read, BufWriter};
use gdk_pixbuf::{Pixbuf, Colorspace, InterpType, PixbufExt};

#[allow(dead_code)]
pub struct Image {
	buffer: Vec<u8>,
	width: usize,
	height: usize,
	one_frame: Duration,
	capturer: Capturer,
}

impl Image {

	#[allow(dead_code)]
	fn flip(buffer: scrap::Frame, width: usize, height: usize) -> Vec<u8> {
		println!("flip");
		// Flip the BGRA image into a RGBA image.
		let mut bitflipped = Vec::with_capacity(width * height * 4);
		let stride = buffer.len() / height;

		for y in 0..height {
			for x in 0..width {
				let i = stride * y + 4 * x;
				bitflipped.extend_from_slice(&[
					buffer[i + 2],
					buffer[i + 1],
					buffer[i],
					255,
				]);
			}
		}
		bitflipped
	}

	#[allow(dead_code)]
	pub fn new() -> Image {
		let display = Display::primary().expect("Couldn't find primary display.");
		let capturer = Capturer::new(display).expect("Couldn't begin capture.");
		let (w, h) = (capturer.width(), capturer.height());
		println!("{} {}", w, h);

		Image {
			buffer: Vec::new(),
			width: w,
			height: h,
			one_frame: Duration::new(0, 0),
			capturer: capturer
		}
	}

	#[allow(dead_code)]
	pub fn set_one_frame(&mut self, fps: u32) {
		let one_second = Duration::new(1, 0);
		let one_frame = one_second / fps;
		println!("frame per second: {}", fps);
		self.one_frame = one_frame;
	}

	#[allow(dead_code)]
	pub fn get_one_frame(&mut self) -> Duration {
		self.one_frame
	}

	#[allow(dead_code)]
	pub fn update(&mut self) {		
		println!("update");
		// buffer is BGRA image
		let buffer = match self.capturer.frame() {
			Ok(buffer) => buffer,
			Err(error) => {
				if error.kind() == WouldBlock {
					// Keep spinning.
					thread::sleep(self.one_frame);
					return;
				} else {
					panic!("Error: {}", error);
				}
			}
		};

		let bitflipped = Image::flip(buffer, self.width, self.height);
		self.buffer = bitflipped;
	}

	#[allow(dead_code)]
	pub fn send(&mut self, writer: &mut BufWriter<TcpStream>) {
		println!("write");
		writer.write(&self.buffer).unwrap();
	}

	#[allow(dead_code)]
	pub fn recv(stream: &mut TcpStream, width: i32, height: i32) -> Vec<u8> {
		println!("recv_images");
		let bufsize = width * height * 4;
		let mut buffer = vec![0u8; bufsize as usize];

		match stream.read_exact(&mut buffer) {
			Ok(_) => {
				println!("Read successfully");
			}
			Err(e) => {
				println!("Error: {}", e);
			}
		};
		buffer
	}

	#[allow(dead_code)]
	pub fn make_pixbuf(buffer: Vec<u8>, width: i32, height: i32) -> Pixbuf {
		let pixbuf = Pixbuf::new_from_vec(
			buffer,
			Colorspace::Rgb,
			true,
			8,
			width,
			height,
			width * 4
		);
		pixbuf
	}

	#[allow(dead_code)]
	pub fn scale_pixbuf(pixbuf: Pixbuf, width: i32, height: i32, ratio: f32) -> Pixbuf {	
		let scaled_pixbuf = pixbuf.scale_simple(
			(width as f32 * ratio) as i32,
			(height as f32 * ratio) as i32,
			InterpType::Bilinear
		).unwrap();
		scaled_pixbuf
	}
}