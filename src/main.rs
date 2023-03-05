mod complex;

use complex::Complex;

use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf;
use ggez::graphics::{self, Color, DrawParam};
use ggez::event::{self, EventHandler};

use palette::{self, FromColor};

const WIDTH: f32 = 500.0;
const HEIGHT: f32 = 500.0;
const SCREEN_SIZE: f32 = WIDTH * HEIGHT;

const FPS: u32 = 60;

const MAX_ITERATIONS: f32 = 100.0;
const MAX_STABLE: f32 = 2.0;

fn main() -> GameResult {
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
fn into_range(val: f32, constant: f32) -> f32 {
	return ((val / constant) * 4.0) - 2.0;
}

fn calculate_for_pixel(x: usize, y: usize) -> Color {
	let mut iterations = 0.0;

	let translated_x = (x as f32 / 2.0) + 65.0;
	let translated_y = (y as f32 / 2.0) + 125.0;
	
	let c = Complex::new(
		into_range(translated_x, WIDTH),
		into_range(translated_y, HEIGHT)
	);

	let mut z = Complex::new(0.01, 0.01);

	while iterations < MAX_ITERATIONS && z.abs() < MAX_STABLE {
		iterations += 1.0;
		z = (z * z) + c;
	}

	let alpha = iterations / MAX_ITERATIONS;

	let hsv = palette::Hsv::new(216.0, 1.0 - alpha, alpha + 0.2);
	let srgb = palette::Srgb::from_color(hsv);

	Color::new(srgb.red, srgb.green, srgb.blue, 1.0)

}

struct MandelbrotViewer {
	batch: graphics::InstanceArray,
}

impl MandelbrotViewer {
	pub fn new(context: &mut Context) -> MandelbrotViewer {
		let mut instance_array = graphics::InstanceArray::new(context, None);
		instance_array.resize(context, SCREEN_SIZE as u32);

		let mut results = Vec::with_capacity(SCREEN_SIZE as usize);

		for x in 0..(WIDTH as usize) {
			for y in 0..(HEIGHT as usize) {
				let pixel_color = calculate_for_pixel(x, y);

				let params = DrawParam::new()
					.dest([x as f32, y as f32])
					.color(pixel_color);

				results.push(params);
			}
		}

		instance_array.set(results);
		
		MandelbrotViewer { 
			batch: instance_array,
		}
	}
}

impl EventHandler for MandelbrotViewer {
	fn update(&mut self, context: &mut Context) -> GameResult {
		while context.time.check_update_time(FPS) {
			// 
		}

		GameResult::Ok(())
	}

	fn draw(&mut self, context: &mut Context) -> GameResult {
		let mut canvas = graphics::Canvas::from_frame(context, Color::BLACK);
		canvas.draw(&self.batch, DrawParam::new());

		canvas.finish(context)?;
		ggez::timer::yield_now();

		GameResult::Ok(())
	}
}
