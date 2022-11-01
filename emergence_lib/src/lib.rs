//! Provides plugins needed by the Emergence game.
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![forbid(unsafe_code)]
#![warn(clippy::doc_markdown)]
use std::marker::PhantomData;

pub mod camera;
pub mod cursor;
pub mod curves;
pub mod hive_mind;
pub mod organisms;
pub mod signals;
pub mod terrain;
pub mod tiles;

/// Marks an enum whose variants can be iterated over in the order they are defined.
pub trait IterableEnum: Sized {
    /// The number of variants of this action type
    const N_VARIANTS: usize;

    /// Iterates over the possible variants in the order they were defined.
    fn variants() -> EnumIter<Self> {
        EnumIter::default()
    }

    /// Returns the default value for the variant stored at the provided index if it exists.
    ///
    /// This is mostly used internally, to enable space-efficient iteration.
    fn get_at(index: usize) -> Option<Self>;

    /// Returns the position in the defining enum of the given action
    fn index(&self) -> usize;
}

/// An iterator of enum variants.
///
/// Created by calling [`IterEnum::iter`].
#[derive(Debug, Clone)]
pub struct EnumIter<A: IterableEnum> {
    /// Keeps track of which variant should be provided next.
    ///
    /// Alternatively, `min(index - 1, 0)` counts how many variants have already been iterated
    /// through.
    index: usize,
    /// Marker used to keep track of which `IterableEnum` this `EnumIter` iterates through.
    ///
    /// For more information, see [`Phantom`].
    _phantom: PhantomData<A>,
}

impl<A: IterableEnum> Iterator for EnumIter<A> {
    type Item = A;

    fn next(&mut self) -> Option<A> {
        let item = A::get_at(self.index);
        if item.is_some() {
            self.index += 1;
        }

        item
    }
}

impl<A: IterableEnum> ExactSizeIterator for EnumIter<A> {
    fn len(&self) -> usize {
        A::N_VARIANTS
    }
}

// We can't derive this, because otherwise it won't work when A is not default
impl<A: IterableEnum> Default for EnumIter<A> {
    fn default() -> Self {
        EnumIter {
            index: 0,
            _phantom: PhantomData::default(),
        }
    }
}
