//! Allocation-less struct to non-lazily filter array iterators

use std::{array, iter};

/// Allocation-less struct to non-lazily filter array iterators
pub struct FilteredArrayIter<T, const N: usize> {
    /// Holds the original array, with filtered elements swapped to the end
    arr: [T; N],
    /// The number of elements not-filtered.
    len: usize,
}

impl<T, const N: usize> From<[T; N]> for FilteredArrayIter<T, N> {
    fn from(arr: [T; N]) -> Self {
        Self { arr, len: N }
    }
}

impl<T, const N: usize> IntoIterator for FilteredArrayIter<T, N> {
    type Item = T;

    type IntoIter = iter::Take<array::IntoIter<T, N>>;

    fn into_iter(self) -> Self::IntoIter {
        self.arr.into_iter().take(self.len)
    }
}

impl<T, const N: usize> FilteredArrayIter<T, N> {
    /// Analogous to [`std::iter::Iterator::filter`], but evaluates
    /// the predicate immediately instead of waiting until we iterate
    /// over the array.
    ///
    /// Does not keep the order of iteration constant.
    ///
    /// Removes elements where predicate(element) = false, but defers
    /// drop until the whole structure is dropped.
    pub fn filter(&mut self, mut predicate: impl FnMut(&T) -> bool) {
        let mut i = 0;
        while i < self.len {
            if predicate(&self.arr[i]) {
                i += 1
            } else {
                self.arr.swap(i, self.len - 1);
                self.len -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::array;

    #[test]
    fn filtered_array() {
        let arr: [usize; 10] = array::from_fn(|i| i);
        let mut filtered_array = FilteredArrayIter::from(arr);
        filtered_array.filter(|i| i % 2 == 0);
        filtered_array.filter(|i| i % 3 == 0);
        let out = filtered_array.into_iter().collect::<Vec<_>>();
        assert_eq!(out, [0, 6]);
    }
}
