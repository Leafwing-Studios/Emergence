use auto_ops::impl_op_ex;
use rand::distributions::{Distribution, Standard, Uniform};
use rand::Rng;
use std::ops;

// We're using a horizontal layout hex grid
// with an "axial coordinate" system
// See: https://www.redblobgames.com/grids/hexagons/
// alpha == q, beta == r from that article

#[derive(Debug)]
pub struct Position {
	pub alpha: isize,
	pub beta: isize,
}

impl Position {
	pub fn dist(self, b: Position) -> isize {
		((self.alpha - b.alpha).abs()
			+ (self.alpha + self.beta - b.alpha - b.beta).abs()
			+ (self.beta - b.beta).abs())
			/ 2
	}
}

impl_op_ex!(+ |a: Position, b: Position| -> Position {
	 Position{
		 alpha: a.alpha + b.alpha,
		 beta: a.beta + b.beta
	}
});

impl_op_ex!(-|a: Position, b: Position| -> Position {
	Position {
		alpha: a.alpha - b.alpha,
		beta: a.beta - b.beta,
	}
});

impl_op_ex!(+= |a: &mut Position, b: &Position| { a.alpha += b.alpha; a.beta += b.beta;});
impl_op_ex!(-= |a: &mut Position, b: &Position| { a.alpha -= b.alpha; a.beta -= b.beta;});

#[derive(Debug)]
pub enum HexDirection {
	East,
	Southeast,
	Southwest,
	West,
	Northwest,
	Northeast,
}

// Generate a random direction with:
// use rand::distributions::Standard;
// let mut rng = &mut rand::thread_rng();
// let direction: HexDirection = rng.sample(Standard);

impl Distribution<HexDirection> for Standard {
	fn sample<R: Rng + ?Sized>(&self, mut rng: &mut R) -> HexDirection {
		let options = Uniform::from(1..=6);
		let choice = options.sample(&mut rng);

		use HexDirection::*;
		match choice {
			1 => East,
			2 => Southeast,
			3 => Southwest,
			4 => West,
			5 => Northwest,
			6 => Northeast,
			_ => unreachable!(),
		}
	}
}

impl HexDirection {
	pub fn offset(self) -> Position {
		use HexDirection::*;
		match self {
			East => Position { alpha: 1, beta: 0 },
			Southeast => Position { alpha: 1, beta: -1 },
			Southwest => Position { alpha: 0, beta: -1 },
			West => Position { alpha: -1, beta: 0 },
			Northwest => Position { alpha: -1, beta: 1 },
			Northeast => Position { alpha: 0, beta: 1 },
		}
	}
}

pub enum ID {
	Empty,
	Ant,
	Plant,
	Fungus,
}

pub enum SignalType {
	Passive(ID),
	Push(ID),
	Pull(ID),
	Work,
}
