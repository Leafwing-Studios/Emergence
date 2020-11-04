use auto_ops::impl_op_ex;
use rand::distributions::{Distribution, Standard, Uniform};
use rand::Rng;
use std::ops;

#[derive(Debug)]
pub struct Position {
	pub x: isize,
	pub y: isize,
}

impl_op_ex!(+ |a: Position, b: Position| -> Position {
	 Position{
		 x: a.x + b.x,
		 y: a.y + b.y
	}
});

impl_op_ex!(-|a: Position, b: Position| -> Position {
	Position {
		x: a.x - b.x,
		y: a.y - b.y,
	}
});

impl_op_ex!(+= |a: &mut Position, b: &Position| { a.x += b.x; a.y += b.y;});
impl_op_ex!(-= |a: &mut Position, b: &Position| { a.x -= b.x; a.y -= b.y;});

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
			5 => Northeast,
			6 => Northeast,
			_ => unreachable!(),
		}
	}
}

impl HexDirection {
	fn offset(self) -> Position {
		use HexDirection::*;
		match self {
			East => Position { x: 1, y: 0 },
			Southeast => Position { x: 1, y: -1 },
			Southwest => Position { x: -1, y: -1 },
			West => Position { x: -1, y: 0 },
			Northeast => Position { x: 1, y: 1 },
			Northwest => Position { x: -1, y: 1 },
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
