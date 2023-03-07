mod complex;

use std::collections::HashMap;

use complex::Complex;

use ggez::input::keyboard::KeyInput;
use ggez::mint::Point2;
use ggez::winit::event::{VirtualKeyCode};
use ggez::{Context, ContextBuilder, GameResult as Result};
use ggez::conf;
use ggez::graphics::{self, Color, DrawParam, InstanceArray};
use ggez::event::{self, EventHandler};

use palette::{self, FromColor};

const WIDTH: f32 = 500.0;
const HEIGHT: f32 = 500.0;
const SCREEN_SIZE: f32 = WIDTH * HEIGHT;

const X_TRANSLATE: f32 = 65.0;
const Y_TRANSLATE: f32 = 125.0;

const FPS: u32 = 30;

const MAX_ITERATIONS: f32 = 100.0;
const MAX_STABLE: f32 = 2.0;

const fn blank_point() -> Point2<f32> {
	Point2 { x: 0.0, y: 0.0 }
}

fn main() -> Result {
	let window_setup = conf::WindowSetup::default()
		.title("Mandelbrot Viewer")
		.vsync(true);

	let window_mode = conf::WindowMode::default()
		.dimensions(WIDTH, HEIGHT)
		.resizable(false);

	let (mut context, event_loop) = ContextBuilder::new("mandelbrot_viewer", "Will")
		.window_setup(window_setup)
		.window_mode(window_mode)
		.build()?;

	let viewer = MandelbrotViewer::new(&mut context);
	event::run(context, event_loop, viewer);
}


// c must be in range -2 < x < 2
fn into_range(val: f32, constant: f32, magnification: f32) -> f32 {
	return (((val / constant) / magnification) * 4.0) - 2.0;
}

struct MovementKeyData {
	is_down: bool,
	velocity: Point2<f32>
}

impl MovementKeyData {
	pub fn new(x_velocity: f32, y_velocity: f32) -> MovementKeyData {
		MovementKeyData {
			is_down: false,
			velocity: Point2 { x: x_velocity, y: y_velocity }
		}
	}
}

struct MandelbrotViewer {
	batch: InstanceArray,

	view_offset: Point2<f32>,
	movement_data: HashMap<VirtualKeyCode, MovementKeyData>,
	magnification: f32,
}

impl MandelbrotViewer {
	pub fn new(context: &mut Context) -> MandelbrotViewer {
		let mut batch = InstanceArray::new(context, None);
		batch.resize(context, SCREEN_SIZE as u32);

		MandelbrotViewer { 
			batch,

			view_offset: blank_point(),
			movement_data: HashMap::from([
				(VirtualKeyCode::W, MovementKeyData::new(0.0, -5.0)),
				(VirtualKeyCode::A, MovementKeyData::new(-5.0, 0.0)),
				(VirtualKeyCode::S, MovementKeyData::new(0.0, 5.0)),
				(VirtualKeyCode::D, MovementKeyData::new(5.0, 0.0)),
			]),

			magnification: 1.0,
		}
	}

	fn calculate_for_pixel(&self, x: usize, y: usize) -> Color {
		let translated_x = (x as f32 / 2.0) + X_TRANSLATE + self.view_offset.x;
		let translated_y = (y as f32 / 2.0) + Y_TRANSLATE + self.view_offset.y;
		
		let c = Complex::new(
			into_range(translated_x, WIDTH, self.magnification),
			into_range(translated_y, HEIGHT, self.magnification)
		);
	
		let mut z = Complex::new(0.01, 0.01);
		let mut iterations = 0.0;
	
		while iterations < MAX_ITERATIONS && z.abs() < MAX_STABLE {
			iterations += 1.0;
			z = (z * z) + c;
		}
	
		let alpha = iterations / MAX_ITERATIONS;
	
		let hsv = palette::Hsv::new(alpha * 360.0, 1.0, 1.0);
		let srgb = palette::Srgb::from_color(hsv);
	
		Color::new(srgb.red, srgb.green, srgb.blue, 1.0)
	
	}
	
	fn construct_batch(&mut self) -> () {
		let mut results = Vec::with_capacity(SCREEN_SIZE as usize);
	
		for x in 0..(WIDTH as usize) {
			for y in 0..(HEIGHT as usize) {
				let pixel_color = self.calculate_for_pixel(x, y);
	
				let params = DrawParam::new()
					.dest([x as f32, y as f32])
					.color(pixel_color);
	
				results.push(params);
			}
		}
	
		self.batch.set(results);
	}
}

impl EventHandler for MandelbrotViewer {
	fn update(&mut self, context: &mut Context) -> Result {
		while context.time.check_update_time(FPS) {
			for (_key, key_data) in self.movement_data.iter() {
				if !key_data.is_down {
					continue;
				}

				self.view_offset.x += key_data.velocity.x;
				self.view_offset.y += key_data.velocity.y;
			}
		}

		self.construct_batch();

		Ok(())
	}

	fn draw(&mut self, context: &mut Context) -> Result {
		let mut canvas = graphics::Canvas::from_frame(context, Color::BLACK);
		canvas.draw(&self.batch, DrawParam::new());

		canvas.finish(context)?;
		ggez::timer::yield_now();

		Ok(())
	}

	fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, repeated: bool) -> Result {
		if !repeated {
			if let Some(keycode) = input.keycode {
				if keycode == VirtualKeyCode::R {
					self.view_offset = blank_point();
					self.magnification = 1.0;
				
				} else if keycode == VirtualKeyCode::O {
					self.magnification += 2.0;

					self.view_offset.x += (WIDTH / 2.0) + 65.0;
					self.view_offset.y += (HEIGHT / 2.0) + 125.0;

				} else if keycode == VirtualKeyCode::P {
					let new_magnification = self.magnification - 2.0;

					if new_magnification >= 1.0 {
						self.magnification = new_magnification;
						self.view_offset.x -= (WIDTH / 2.0) + 65.0;
						self.view_offset.y -= (HEIGHT / 2.0) + 125.0;
					}

				} else if let Some(key_data) = self.movement_data.get_mut(&keycode) {
					key_data.is_down = true;
				}
			}
		}

		Ok(())
	}

	fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> Result {
		if let Some(keycode) = input.keycode {

			if let Some(key_data) = self.movement_data.get_mut(&keycode) {
				key_data.is_down = false;
			}
		}

		Ok(())
	}
}
