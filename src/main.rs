#![windows_subsystem = "windows"]

mod complex;

use std::collections::HashMap;
use std::thread;

use complex::Complex;

use ggez::input::keyboard::KeyInput;
use ggez::mint::Point2;
use ggez::winit::event::VirtualKeyCode;
use ggez::{Context, ContextBuilder, GameResult as Result};
use ggez::conf;
use ggez::graphics::{self, Color, DrawParam, InstanceArray};
use ggez::event::{self, EventHandler};

use palette::{self, FromColor};

const WIDTH: f32 = 500.0;
const HEIGHT: f32 = 500.0;
const SCREEN_SIZE: f32 = WIDTH * HEIGHT;

const FPS: u32 = 144;

const MAX_ITERATIONS: f64 = 100.0;
const MAX_STABLE: f64 = 2.0;

const THREADS: usize = 10;

fn main() -> Result {
	let window_setup = conf::WindowSetup::default()
		.title("Mandelbrot Viewer")
		.vsync(true);

	let window_mode = conf::WindowMode::default()
		.dimensions(WIDTH, HEIGHT)
		.resizable(false);

	let (mut context, event_loop) = ContextBuilder::new("mandelbrot_viewer", "ReturnedTrue")
		.window_setup(window_setup)
		.window_mode(window_mode)
		.build()?;

	let viewer = MandelbrotViewer::new(&mut context);
	event::run(context, event_loop, viewer);
}

const fn blank_point() -> Point2<f64> {
	Point2 { x: 0.0, y: 0.0 }
}

#[inline]
fn into_range(value: f64, constant: f64, magnification: f64) -> f64 {
	return (((value / constant) / magnification) * 4.0) - 2.0;
}

fn calculate_for_pixel(x: usize, y: usize, view_offset: Point2<f64>, magnification: f64) -> Color {
	let translated_x = x as f64 + view_offset.x;
	let translated_y = y as f64 + view_offset.y;
	
	let c = Complex::new(
		into_range(translated_x, WIDTH as f64, magnification),
		into_range(translated_y, HEIGHT as f64, magnification)
	);

	let mut z = Complex::new(0.01, 0.01);
	let mut iterations = 0.0;

	while z.abs() < MAX_STABLE {
		if iterations > MAX_ITERATIONS {
			return Color::new(0.0, 0.0, 0.0, 1.0);
		}

		iterations += 1.0;
		z = (z * z) + c;
	}

	let alpha = iterations / MAX_ITERATIONS;

	let hsv = palette::Hsv::new(alpha as f32 * 360.0, 1.0, 1.0);
	let srgb = palette::Srgb::from_color(hsv);

	Color::new(srgb.red, srgb.green, srgb.blue, 1.0)

}

fn calculate_for_range(x_start: usize, x_end: usize, view_offset: Point2<f64>, magnification: f64) -> Vec<DrawParam> {
	let mut range_results = Vec::with_capacity((x_end - x_start) * (HEIGHT as usize));

	for x in x_start..x_end {
		for y in 0..(HEIGHT as usize) {
			let pixel_color = calculate_for_pixel(x, y, view_offset, magnification);

			let params = DrawParam::new()
				.dest([x as f32, y as f32])
				.color(pixel_color);

			range_results.push(params);
		}
	}

	range_results
}

struct MovementKeyData {
	is_down: bool,
	velocity: Point2<f64>
}

impl MovementKeyData {
	pub fn new(x_velocity: f64, y_velocity: f64) -> MovementKeyData {
		MovementKeyData {
			is_down: false,
			velocity: Point2 { x: x_velocity, y: y_velocity }
		}
	}
}

struct MandelbrotViewer {
	batch: InstanceArray,

	movement_data: HashMap<VirtualKeyCode, MovementKeyData>,

	has_parameters_changed: bool,
	view_offset: Point2<f64>,
	magnification: f64,
}

impl MandelbrotViewer {
	pub fn new(context: &mut Context) -> MandelbrotViewer {
		let mut batch = InstanceArray::new(context, None);
		batch.resize(context, SCREEN_SIZE as u32);

		MandelbrotViewer { 
			batch,

			movement_data: HashMap::from([
				(VirtualKeyCode::W, MovementKeyData::new(0.0, -10.0)),
				(VirtualKeyCode::A, MovementKeyData::new(-10.0, 0.0)),
				(VirtualKeyCode::S, MovementKeyData::new(0.0, 10.0)),
				(VirtualKeyCode::D, MovementKeyData::new(10.0, 0.0)),
			]),

			// In order to invoke first render
			has_parameters_changed: true,
			view_offset: blank_point(),
			magnification: 1.0,
		}
	}

	fn construct_batch(&mut self) -> () {
		let mut results = Vec::with_capacity(SCREEN_SIZE as usize);
		let mut threads = Vec::with_capacity(THREADS);

		let mut accumulated_x = 0;
		let per_thread_x = (WIDTH as usize) / THREADS;

		for _ in 0..THREADS {
			let acc = accumulated_x;
			let offset = self.view_offset;
			let mag = self.magnification;

			let t = thread::spawn(move || calculate_for_range(acc, acc + per_thread_x, offset, mag));
			threads.push(t);

			accumulated_x += per_thread_x;
		}

		for t in threads {
			for params in t.join().expect("thread panicked") {
				results.push(params);
			}
		}
		
		self.batch.set(results);
	}
}

impl EventHandler for MandelbrotViewer {
	fn update(&mut self, context: &mut Context) -> Result {
		while context.time.check_update_time(FPS) {
			let delta_time = context.time.delta().as_secs_f64();

			for (_key, key_data) in self.movement_data.iter() {
				if !key_data.is_down {
					continue;
				}

				self.view_offset.x += key_data.velocity.x * delta_time;
				self.view_offset.y += key_data.velocity.y * delta_time;

				self.has_parameters_changed = true;
			}
		}

		if self.has_parameters_changed {
			self.construct_batch();
			self.has_parameters_changed = false;
		}
		
		Ok(())
	}

	fn draw(&mut self, context: &mut Context) -> Result {
		let mut canvas = graphics::Canvas::from_frame(context, Color::BLACK);
		canvas.draw(&self.batch, DrawParam::new());

		canvas.finish(context)?;
		ggez::timer::yield_now();

		Ok(())
	}

	fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, repeated: bool) -> Result {
		if repeated {
			return Ok(())
		}

		if let Some(keycode) = input.keycode {
			if let Some(key_data) = self.movement_data.get_mut(&keycode) {
				key_data.is_down = true;

			} else {
				match keycode {
					VirtualKeyCode::R => {
						self.view_offset = blank_point();
						self.magnification = 1.0;
						self.has_parameters_changed = true;
					},

					VirtualKeyCode::E => {
						let old_mag = self.magnification;
						let new_mag = 2.0 * old_mag;
	
						let mouse_pos = ctx.mouse.position();
						let offset = self.view_offset;

						let pivot_x = (offset.x + mouse_pos.x as f64) / old_mag * new_mag;
						let pivot_y = (offset.y + mouse_pos.y as f64) / old_mag * new_mag;

						self.magnification = new_mag;

						self.view_offset.x = pivot_x - (WIDTH as f64) / 2.0;
						self.view_offset.y = pivot_y - (HEIGHT as f64) / 2.0;
	
						self.has_parameters_changed = true;
					},

					VirtualKeyCode::Q => {
						let old_mag = self.magnification;
						let new_mag = (0.5 * old_mag).max(1.0);

						let mouse_pos = ctx.mouse.position();
						let offset = self.view_offset;

						let pivot_x = (offset.x + mouse_pos.x as f64) / old_mag * new_mag;
						let pivot_y = (offset.y + mouse_pos.y as f64) / old_mag * new_mag;

						self.magnification = new_mag;

						self.view_offset.x = pivot_x - (WIDTH as f64) / 2.0;
						self.view_offset.y = pivot_y - (HEIGHT as f64) / 2.0;
	
						self.has_parameters_changed = true;
					}
					_ => {}
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
