use ndarray::prelude::*;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ID {
	Ant,
	Plant,
	Fungus,
}

#[derive(Debug, Clone, Copy)]
pub enum SignalType {
	Passive(ID),
	Push(ID),
	Pull(ID),
	Work,
}

pub struct Contents {
	pub id: Array2<Option<ID>>,
}

impl Deref for Contents {

	type Target = Array2<Option<ID>>;

	fn deref(&self) -> &Self::Target {
		&self.id
	}
}

impl DerefMut for Contents {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.id
	}
}

