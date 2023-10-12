//! A helpful trait to allow us to iterate over all variants of an enum type

use std::marker::PhantomData;

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
/// Created by calling [`IterableEnum::variants`].
#[derive(Debug, Clone)]
pub struct EnumIter<A: IterableEnum> {
    /// Keeps track of which variant should be provided next.
    ///
    /// Alternatively, `min(index - 1, 0)` counts how many variants have already been iterated
    /// through.
    index: usize,
    /// Marker used to keep track of which `IterableEnum` this `EnumIter` iterates through.
    ///
    /// For more information, see [`PhantomData`](std::marker::PhantomData).
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
            _phantom: PhantomData,
        }
    }
}
