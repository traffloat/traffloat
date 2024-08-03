//! Common units to describe liquids.

use std::ops;

use derive_more::{Add, AddAssign, From, Neg, Sub, SubAssign, Sum};
use serde::{Deserialize, Serialize};

macro_rules! define_unit {
    (
        $(
            $(#[$meta:meta])*
            $vis:vis $ident:ident;
        )*
    ) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
            #[derive(From, Add, AddAssign, Sub, SubAssign, Sum, Neg)]
            #[derive(Serialize, Deserialize)]
            $vis struct $ident {
                /// Unit quantity.
                pub quantity: f32,
            }

            impl ops::Mul<f32> for $ident {
                type Output = Self;

                fn mul(mut self, other: f32) -> Self {
                    self.quantity *= other;
                    self
                }
            }

            impl ops::Div<f32> for $ident {
                type Output = Self;

                fn div(mut self, other: f32) -> Self {
                    self.quantity /= other;
                    self
                }
            }
         )*
    }
}

define_unit! {
    /// The mass of fluid.
    pub Mass;

    /// The space occupied by fluid.
    pub Volume;

    /// The force a fluid mixture exerts on its container.
    pub Pressure;

    /// The viscosity of a liquid, inversely proportional to flow rate.
    pub Viscosity;

    /// Mass per volume.
    pub Density;

    /// Volume per mass, a reciprocal of density.
    pub SpecificVolume;
}

macro_rules! operators {
    ($($left:ident * $right:ident = $out:ident;)*) => {
        $(
            impl ops::Mul<$right> for $left {
                type Output = $out;

                fn mul(self, other: $right) -> $out {
                    $out { quantity: self.quantity * other.quantity }
                }
            }
            impl ops::Mul<$left> for $right {
                type Output = $out;

                fn mul(self, other: $left) -> $out {
                    $out { quantity: self.quantity * other.quantity }
                }
            }
            impl ops::Div<$left> for $out {
                type Output = $right;

                fn div(self, other: $left) -> $right {
                    $right { quantity: self.quantity / other.quantity }
                }
            }
            impl ops::Div<$right> for $out {
                type Output = $left;

                fn div(self, other: $right) -> $left {
                    $left { quantity: self.quantity / other.quantity }
                }
            }
        )*
    }
}

operators! {
    Mass * SpecificVolume = Volume;
    Density * Volume = Mass;
}
