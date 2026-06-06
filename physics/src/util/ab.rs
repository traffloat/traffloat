use std::ops;

use bevy::reflect::{self, FromReflect, GetTypeRegistration, Reflect};
use traffloat_proto::proto;

use crate::util::{MergeSortedItem, merge_sorted};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Reflect)]
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

    pub fn zip<U>(self, other: AlphaBeta<U>) -> AlphaBeta<(T, U)> {
        AlphaBeta { alpha: (self.alpha, other.alpha), beta: (self.beta, other.beta) }
    }

    pub fn bimap<U, V>(self, other: AlphaBeta<U>, mut f: impl FnMut(T, U) -> V) -> AlphaBeta<V> {
        AlphaBeta { alpha: f(self.alpha, other.alpha), beta: f(self.beta, other.beta) }
    }
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
    pub fn net_diff(self) -> T::Output { self.alpha - self.beta }

    pub fn atob(self) -> T::Output { self.beta - self.alpha }
}

impl AlphaBeta<f32> {
    #[must_use]
    pub fn harmonic_mean(self) -> f32 {
        let sum = self.sum();
        if sum == 0.0 {
            // avoid division by zero
            0.0
        } else {
            let product = self.alpha * self.beta;
            product / sum
        }
    }
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

impl<'a, T> AlphaBeta<&'a [T]> {
    pub fn zip_iter(self) -> impl Iterator<Item = AlphaBeta<&'a T>> {
        self.alpha.iter().zip(self.beta.iter()).map(|(alpha, beta)| AlphaBeta { alpha, beta })
    }
}

impl<'a, T: ops::Deref> AlphaBeta<&'a T> {
    pub fn as_deref(self) -> AlphaBeta<&'a T::Target> {
        AlphaBeta { alpha: &**self.alpha, beta: &**self.beta }
    }
}

impl<'a, T> AlphaBeta<&'a mut [T]> {
    pub fn zip_iter_mut(self) -> impl Iterator<Item = AlphaBeta<&'a mut T>> {
        self.alpha
            .iter_mut()
            .zip(self.beta.iter_mut())
            .map(|(alpha, beta)| AlphaBeta { alpha, beta })
    }
}

impl<A, B> AlphaBeta<(A, B)> {
    pub fn unzip(self) -> (AlphaBeta<A>, AlphaBeta<B>) {
        (
            AlphaBeta { alpha: self.alpha.0, beta: self.beta.0 },
            AlphaBeta { alpha: self.alpha.1, beta: self.beta.1 },
        )
    }
}

pub trait Which:
    Default
    + Copy
    + Send
    + Sync
    + Reflect
    + FromReflect
    + GetTypeRegistration
    + reflect::Typed
    + 'static
{
    type Other: Which;
    fn other(self) -> Self::Other { Self::Other::default() }

    fn select<T>(self, ab: AlphaBeta<T>) -> T;
    fn select_ref<T>(self, ab: &AlphaBeta<T>) -> &T;
    fn select_mut<T>(self, ab: &mut AlphaBeta<T>) -> &mut T;

    fn get<T>(self, ab: impl GetAb<T>) -> T;

    fn proto(self) -> proto::AlphaOrBeta;
}

macro_rules! define_which {
    (
        $ident:ident, $field:ident, $variant:ident, $other:ident, $get:ident
    ) => {
        #[derive(Default, Clone, Copy, Reflect)]
        pub struct $ident;

        impl Which for $ident {
            type Other = $other;

            fn select<T>(self, ab: AlphaBeta<T>) -> T { ab.$field }
            fn select_ref<T>(self, ab: &AlphaBeta<T>) -> &T { &ab.$field }
            fn select_mut<T>(self, ab: &mut AlphaBeta<T>) -> &mut T { &mut ab.$field }

            fn get<T>(self, ab: impl GetAb<T>) -> T { ab.$get() }

            fn proto(self) -> proto::AlphaOrBeta { proto::AlphaOrBeta::$variant }
        }
    };
}

define_which!(Alpha, alpha, Alpha, Beta, alpha);
define_which!(Beta, beta, Beta, Alpha, beta);

pub trait GetAb<T> {
    fn alpha(self) -> T;
    fn beta(self) -> T;
}
