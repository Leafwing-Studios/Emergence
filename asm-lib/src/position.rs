use auto_ops::impl_op_ex;
use rand::distributions::{Distribution, Standard, Uniform};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

use crate::config::MAP_SIZE;
use crate::entity_map::EntityMap;

#[derive(Debug, Clone, Copy)]
pub struct CubePosition {
	pub alpha: isize,
	pub beta: isize,
	pub gamma: isize,
}

impl CubePosition {
	pub fn to_axial(self) -> Position {
		Position {
			alpha: self.alpha,
			beta: self.beta,
		}
	}
}

// We're using a horizontal layout hex grid
// with an "axial coordinate" system
// See: https://www.redblobgames.com/grids/hexagons/
// alpha == q, beta == r from that article

static ORIGIN: Position = Position { alpha: 0, beta: 0 };
static NEIGHBORS: [Position; 6] = [
	Position { alpha: 1, beta: 0 },
	Position { alpha: 1, beta: -1 },
	Position { alpha: 0, beta: -1 },
	Position { alpha: -1, beta: 0 },
	Position { alpha: -1, beta: 1 },
	Position { alpha: 0, beta: 1 },
];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Position {
	pub alpha: isize,
	pub beta: isize,
}

// Core methods
impl Position {
	pub fn inbounds(self) -> bool {
		self.dist(ORIGIN) <= MAP_SIZE
	}

	pub fn dist(self, b: Position) -> isize {
		let (a, b) = (CubePosition::from(self), CubePosition::from(b));
		((a.alpha - b.alpha).abs() + (a.beta - b.beta).abs() + (a.gamma - b.gamma).abs()) / 2
	}

	pub fn translate(self, direction: &HexDirection, distance: isize) -> Position {
		self + direction.offset() * distance
	}
}

// Navigation methods
impl Position {
	pub fn neighbors(self) -> Vec<Position> {
		NEIGHBORS
			.iter()
			.map(|&p| self + p)
			.filter(|p| p.inbounds())
			.collect()
	}

	pub fn neighbors_where(self, f: impl Fn(&Position) -> bool) -> Vec<Position> {
		NEIGHBORS
			.iter()
			.map(|&p| self + p)
			.filter(|p| p.inbounds() && f(p))
			.collect()
	}

	pub fn random_neighbor(self) -> Option<Position> {
		self.neighbors().choose(&mut thread_rng()).copied()
	}
}

// Conversion methods
impl From<Position> for CubePosition {
	fn from(p: Position) -> Self {
		CubePosition {
			alpha: p.alpha,
			beta: p.beta,
			gamma: -p.alpha - p.beta,
		}
	}
}

// Generation methods
impl Position {
	pub fn ring(self, radius: isize) -> Vec<Position> {
		let mut positions: Vec<Position> = Vec::new();

		if radius == 0 {
			positions.push(self);
			return positions;
		}

		let mut current_position = self.translate(&HexDirection::East, radius);

		let mut current_direction = HexDirection::Southwest;

		for _ in 0..6 {
			for _ in 0..radius {
				positions.push(current_position);
				current_position = current_position.translate(&current_direction, 1);
			}

			current_direction = current_direction.rotate(1);
		}
		return positions;
	}

	pub fn hexagon(self, radius: isize) -> Vec<Position> {
		let mut positions: Vec<Position> = Vec::new();

		for i in 0..=radius {
			positions.extend(Position::ring(self, i));
		}

		return positions;
	}
}

impl_op_ex!(+ |a: Position, b: Position| -> Position {
	 Position{
		 alpha: a.alpha + b.alpha,
		 beta: a.beta + b.beta
	}
});

impl_op_ex!(*|a: Position, c: usize| -> Position {
	Position {
		alpha: a.alpha * c as isize,
		beta: a.beta * c as isize,
	}
});

impl_op_ex!(*|c: usize, a: Position| -> Position {
	Position {
		alpha: a.alpha * c as isize,
		beta: a.beta * c as isize,
	}
});

impl_op_ex!(*|a: Position, c: isize| -> Position {
	Position {
		alpha: a.alpha * c,
		beta: a.beta * c,
	}
});

impl_op_ex!(*|c: isize, a: Position| -> Position {
	Position {
		alpha: a.alpha * c,
		beta: a.beta * c,
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
		let options = Uniform::from(0..5);
		let choice = options.sample(&mut rng);

		HexDirection::from_int(choice)
	}
}

impl HexDirection {
	fn from_int(choice: isize) -> HexDirection {
		let int_direction = choice.rem_euclid(6);
		use HexDirection::*;
		match int_direction {
			0 => East,
			1 => Southeast,
			2 => Southwest,
			3 => West,
			4 => Northwest,
			5 => Northeast,
			_ => unreachable!(),
		}
	}

	fn to_int(self) -> u8 {
		use HexDirection::*;
		match self {
			East => 0,
			Southeast => 1,
			Southwest => 2,
			West => 3,
			Northwest => 4,
			Northeast => 5,
		}
	}
	pub fn offset(&self) -> Position {
		use HexDirection::*;
		match self {
			East => NEIGHBORS[0],
			Southeast => NEIGHBORS[1],
			Southwest => NEIGHBORS[2],
			West => NEIGHBORS[3],
			Northwest => NEIGHBORS[4],
			Northeast => NEIGHBORS[5],
		}
	}

	// Positive steps rotates clockwise, negative steps rotate counterclockwise
	pub fn rotate(self, steps: isize) -> HexDirection {
		HexDirection::from_int(self.to_int() as isize + steps.rem_euclid(6))
	}
}
