// FIXME: re-enable missing doc checks
//#![deny(missing_docs)]
//#![deny(clippy::missing_docs_in_private_items)]
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

pub trait IterableEnum: Sized {
    /// The number of variants of this action type
    const N_VARIANTS: usize;

    /// Iterates over the possible actions in the order they were defined
    fn variants() -> EnumIter<Self> {
        EnumIter::default()
    }

    /// Returns the default value for the action stored at the provided index if it exists
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
    index: usize,
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
