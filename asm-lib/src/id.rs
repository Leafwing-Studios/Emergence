use ndarray::prelude::*;

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

pub type Contents = Array2<Option<ID>>;
