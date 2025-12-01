use anchor_lang::prelude::*;

/// A fixed-point numeric type for precise calculations
/// Represents a number as a 128-bit integer with implied decimal point
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Numeric(i128);

impl Numeric {
    pub const ZERO: Numeric = Numeric(0);
    pub const ONE: Numeric = Numeric(1_000_000_000_000); // 1.0 with 12 decimal places
    const SCALE: i128 = 1_000_000_000_000; // 10^12 for precision

    pub fn from_u64(value: u64) -> Self {
        Numeric((value as i128) * Self::SCALE)
    }

    pub fn from_fraction(numerator: u64, denominator: u64) -> Self {
        if denominator == 0 {
            return Numeric::ZERO;
        }
        let result = ((numerator as i128) * Self::SCALE) / (denominator as i128);
        Numeric(result)
    }

    pub fn to_u64(self) -> u64 {
        (self.0 / Self::SCALE) as u64
    }
}

impl std::ops::Add for Numeric {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Numeric(self.0 + other.0)
    }
}

impl std::ops::Sub for Numeric {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Numeric(self.0 - other.0)
    }
}

impl std::ops::Mul for Numeric {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Numeric((self.0 * other.0) / Self::SCALE)
    }
}

impl std::ops::AddAssign for Numeric {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl std::ops::SubAssign for Numeric {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl Default for Numeric {
    fn default() -> Self {
        Numeric::ZERO
    }
}
