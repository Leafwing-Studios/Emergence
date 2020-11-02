#[derive(Debug)]
pub struct Position {
	pub x: isize,
	pub y: isize
}

pub enum ID {
	Empty,
	Ant,
	Plant,
	Fungus
}

pub enum SignalType {
	Passive (ID),
	Push (ID),
	Pull (ID),
	Work
}