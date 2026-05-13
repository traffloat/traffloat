use std::{cmp, iter, ops};

use crate::util::{MergeSortedItem, merge_sorted};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AlphaBeta<T> {
    pub alpha: T,
    pub beta:  T,
}

impl<T> AlphaBeta<T> {
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> AlphaBeta<U> {
        AlphaBeta { alpha: f(self.alpha), beta: f(self.beta) }
    }

    pub fn as_ref(&self) -> AlphaBeta<&T> { AlphaBeta { alpha: &self.alpha, beta: &self.beta } }

    pub fn as_mut(&mut self) -> AlphaBeta<&mut T> {
        AlphaBeta { alpha: &mut self.alpha, beta: &mut self.beta }
    }

    pub fn alpha_if(self, alpha: bool) -> T { if alpha { self.alpha } else { self.beta } }

    pub fn beta_if(self, beta: bool) -> T { if beta { self.beta } else { self.alpha } }
}

impl<T> AlphaBeta<Option<T>> {
    pub fn transpose(self) -> Option<AlphaBeta<T>> {
        match (self.alpha, self.beta) {
            (Some(alpha), Some(beta)) => Some(AlphaBeta { alpha, beta }),
            _ => None,
        }
    }
}

impl<T: ops::Add> AlphaBeta<T> {
    pub fn sum(self) -> T::Output { self.alpha + self.beta }
}

impl<T: ops::Sub> AlphaBeta<T> {
    /// alpha + result = beta
    pub fn atob(self) -> T::Output { self.beta - self.alpha }

    /// beta + result = alpha
    pub fn btoa(self) -> T::Output { self.alpha - self.beta }
}

impl<T: IntoIterator> AlphaBeta<T> {
    /// Merges two iterators with sorted and strictly increasing `key`s
    /// into a single iterator that yields items in sorted order by `key`.
    pub fn merge_sorted<K: Ord>(
        self,
        key: impl Fn(&T::Item) -> K + Copy,
    ) -> impl Iterator<Item = MergeSortedItem<T::Item, T::Item>> {
        merge_sorted(self.alpha, self.beta, key, key)
    }
}
