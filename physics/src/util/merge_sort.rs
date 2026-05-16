use std::{cmp, iter};

use crate::util::AlphaBeta;

/// Merges two iterators with sorted and strictly increasing `key`s
/// into a single iterator that yields items in sorted order by `key`.
pub fn merge_sorted<T1, T2, K: Ord>(
    left: impl IntoIterator<Item = T1>,
    right: impl IntoIterator<Item = T2>,
    mut left_key: impl FnMut(&T1) -> K,
    mut right_key: impl FnMut(&T2) -> K,
) -> impl Iterator<Item = MergeSortedItem<T1, T2>> {
    let mut left = left.into_iter().peekable();
    let mut right = right.into_iter().peekable();

    iter::from_fn(move || match (left.peek(), right.peek()) {
        (Some(a), Some(b)) => match left_key(a).cmp(&right_key(b)) {
            // Only one side has this key, yield and advance this side only.
            cmp::Ordering::Less => left.next().map(MergeSortedItem::Left),
            cmp::Ordering::Greater => right.next().map(MergeSortedItem::Right),
            // Both sides have this key.
            cmp::Ordering::Equal => {
                let a = left.next().unwrap();
                let b = right.next().unwrap();
                Some(MergeSortedItem::Both(a, b))
            }
        },
        (Some(_), None) => left.next().map(MergeSortedItem::Left),
        (None, Some(_)) => right.next().map(MergeSortedItem::Right),
        (None, None) => None,
    })
}

#[derive(Debug, Clone, Copy)]
pub enum MergeSortedItem<T1, T2> {
    Left(T1),
    Right(T2),
    Both(T1, T2),
}

impl<T> MergeSortedItem<T, T> {
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> MergeSortedItem<U, U> {
        match self {
            MergeSortedItem::Left(v) => MergeSortedItem::Left(f(v)),
            MergeSortedItem::Right(v) => MergeSortedItem::Right(f(v)),
            MergeSortedItem::Both(a, b) => MergeSortedItem::Both(f(a), f(b)),
        }
    }

    pub fn any<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        match self {
            MergeSortedItem::Left(v) | MergeSortedItem::Right(v) | MergeSortedItem::Both(v, _) => {
                f(v)
            }
        }
    }
}

impl<T1, T2> MergeSortedItem<T1, T2> {
    pub fn as_ref(&self) -> MergeSortedItem<&T1, &T2> {
        match self {
            MergeSortedItem::Left(v) => MergeSortedItem::Left(v),
            MergeSortedItem::Right(v) => MergeSortedItem::Right(v),
            MergeSortedItem::Both(a, b) => MergeSortedItem::Both(a, b),
        }
    }

    pub fn as_mut(&mut self) -> MergeSortedItem<&mut T1, &mut T2> {
        match self {
            MergeSortedItem::Left(v) => MergeSortedItem::Left(v),
            MergeSortedItem::Right(v) => MergeSortedItem::Right(v),
            MergeSortedItem::Both(a, b) => MergeSortedItem::Both(a, b),
        }
    }
}

impl<T: Default> MergeSortedItem<T, T> {
    pub fn default_ab(self) -> AlphaBeta<T> {
        match self {
            MergeSortedItem::Left(alpha) => AlphaBeta { alpha, beta: T::default() },
            MergeSortedItem::Right(beta) => AlphaBeta { alpha: T::default(), beta },
            MergeSortedItem::Both(alpha, beta) => AlphaBeta { alpha, beta },
        }
    }
}
