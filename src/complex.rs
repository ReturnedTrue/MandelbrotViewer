use std::ops::{Add, Mul};

// Complex numbers are treated like 2D vectors

#[derive(Copy, Clone)]
pub struct Complex {
	real: f32,
	imaginary: f32
}

impl Complex {
	pub fn new(real: f32, imaginary: f32) -> Complex {
		Complex {
			real,
			imaginary,
		}
	}

	// Pythagoras
	pub fn abs(&self) -> f32 {
		((self.real * self.real) + (self.imaginary * self.imaginary)).abs()
	}
}

impl Add<Complex> for Complex {
	type Output = Complex;

	fn add(self, rhs: Complex) -> Self::Output {
		Complex {
			real: self.real + rhs.real,
			imaginary: self.imaginary + rhs.imaginary
		}
	}
}

impl Mul<Complex> for Complex {
	type Output = Complex;

	// (a + bi)(c + di) = (ac - bd) + (ad + bc)i
	fn mul(self, rhs: Complex) -> Self::Output {
		Complex {
			real: (self.real * rhs.real) - (self.imaginary * rhs.imaginary),
			imaginary: (self.real * rhs.imaginary) + (self.imaginary * rhs.real)
		}
	}
}