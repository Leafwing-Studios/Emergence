#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ID {
    Unspecified,
    Ant,
    Plant,
    Fungus,
}

impl Default for ID {
    fn default() -> Self {
        ID::Unspecified
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SignalType {
    Passive(ID),
    Push(ID),
    Pull(ID),
    Work,
}
