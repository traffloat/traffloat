macro_rules! units {
    (
        $(#[$blanket_meta:meta])*
        $blanket:ident($($super:tt)*);
        #[$derive:meta] $base:ty:
        $(
            $(#[$meta:meta])*
            $tys:ident($fmt:literal);
        )*
        $(
            <$a:path> * <$b:path> = $c:path;
        )*
    ) => {
        $(#[$blanket_meta])*
        pub trait $blanket : $($super)* {
            /// Returns the raw value of this struct.
            fn value(self) -> $base;

            /// Returns a reference to the raw value of this struct.
            fn value_mut(&mut self) -> &mut $base;
        }

        $(
            $(#[$meta])*
            #[$derive]
            pub struct $tys(pub $base);

            impl From<$base> for $tys {
                #[inline(always)]
                fn from(base: $base) -> Self {
                    Self(base)
                }
            }

            impl $tys {
                /// Returns the raw value of this struct.
                #[inline(always)]
                pub fn value(self) -> $base {
                    self.0
                }
            }

            impl ::std::fmt::Display for $tys {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    write!(f, $fmt, self.0)
                }
            }

            impl $blanket for $tys {
                #[inline(always)]
                fn value(self) -> $base {
                    self.0
                }

                #[inline(always)]
                fn value_mut(&mut self) -> &mut $base {
                    &mut self.0
                }
            }

            impl ::std::ops::Add for $tys {
                type Output = Self;

                #[inline(always)]
                fn add(self, other: Self) -> Self {
                    $tys(self.value() + other.value())
                }
            }

            impl ::std::ops::AddAssign for $tys {
                #[inline(always)]
                fn add_assign(&mut self, other: Self) {
                    self.0 += other.value();
                }
            }

            impl ::std::ops::Sub for $tys {
                type Output = Self;

                #[inline(always)]
                fn sub(self, other: Self) -> Self {
                    $tys(self.value() - other.value())
                }
            }

            impl ::std::ops::SubAssign for $tys {
                #[inline(always)]
                fn sub_assign(&mut self, other: Self) {
                    self.0 -= other.value();
                }
            }

            impl ::std::ops::Mul<$base> for $tys {
                type Output = Self;

                #[inline(always)]
                fn mul(self, other: $base) -> Self {
                    Self(self.0 * other)
                }
            }

            impl ::std::ops::MulAssign<$base> for $tys {
                #[inline(always)]
                fn mul_assign(&mut self, other: $base) {
                    self.0 *= other;
                }
            }

            impl ::std::ops::DivAssign<$base> for $tys {
                #[inline(always)]
                fn div_assign(&mut self, other: $base) {
                    self.0 /= other;
                }
            }

            impl ::std::ops::Rem for $tys {
                type Output = Self;

                #[inline(always)]
                fn rem(self, other: Self) -> Self {
                    $tys(self.value() % other.value())
                }
            }

            impl ::std::ops::RemAssign for $tys {
                #[inline(always)]
                fn rem_assign(&mut self, other: Self) {
                    self.0 %= other.value();
                }
            }

            impl ::std::iter::Sum for $tys {
                fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                    iter.fold(Self::default(), |a, b| a + b)
                }
            }
        )*

        $(
            impl ::std::ops::Mul<$b> for $a {
                type Output = $c;

                #[inline(always)]
                fn mul(self, other: $b) -> $c {
                    $c(self.value() * other.value())
                }
            }

            impl ::std::ops::Mul<$a> for $b {
                type Output = $c;

                #[inline(always)]
                fn mul(self, other: $a) -> $c {
                    $c(self.value() * other.value())
                }
            }

            impl ::std::ops::Div<$b> for $c {
                type Output = $a;

                #[inline(always)]
                fn div(self, other: $b) -> $a {
                    $a(self.value() / other.value())
                }
            }

            impl ::std::ops::Div<$a> for $c {
                type Output = $b;

                #[inline(always)]
                fn div(self, other: $a) -> $b {
                    $b(self.value() / other.value())
                }
            }
        )*
    };
}

#[cfg(test)]
mod tests {
    units! {
        Blanket(std::fmt::Debug + Clone + Copy + Default + PartialEq + PartialOrd);

        #[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)] f64:
        Accel("{} ms^-2"); Veloc("{} ms^-1"); Length("{} m"); Time("{} m"); Mass("{} kg"); Force("{} N"); Energy("{} J");

        <Accel> * <Time> = Veloc;
        <Veloc> * <Time> = Length;
        <Mass> * <Accel> = Force;
        <Force> * <Length> = Energy;
    }
}
