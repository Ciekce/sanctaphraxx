/*
 * Sanctaphraxx, a UAI Ataxx engine
 * Copyright (C) 2023 Ciekce
 *
 * Sanctaphraxx is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Sanctaphraxx is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Sanctaphraxx. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::core::Square;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AtaxxMove {
    None,
    Null,
    Single(Square),
    Double(Square, Square),
}

impl AtaxxMove {
    #[must_use]
    pub fn pack(&self) -> PackedMove {
        PackedMove::pack(*self)
    }
}

pub enum MoveStrError {
    InvalidFrom,
    InvalidTo,
    WrongSize,
}

impl FromStr for AtaxxMove {
    type Err = MoveStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "0000" {
            return Ok(AtaxxMove::Null);
        }

        match s.len() {
            2 => {
                if let Ok(sq) = Square::from_str(&s[0..2]) {
                    Ok(Self::Single(sq))
                } else {
                    Err(MoveStrError::InvalidTo)
                }
            }
            4 => {
                if let Ok(from) = Square::from_str(&s[0..2]) {
                    if let Ok(to) = Square::from_str(&s[2..4]) {
                        Ok(Self::Double(from, to))
                    } else {
                        Err(MoveStrError::InvalidTo)
                    }
                } else {
                    Err(MoveStrError::InvalidFrom)
                }
            }
            _ => Err(MoveStrError::WrongSize),
        }
    }
}

impl Display for AtaxxMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            AtaxxMove::None => write!(f, "<none>"),
            AtaxxMove::Null => write!(f, "0000"),
            AtaxxMove::Single(to) => write!(f, "{}", to),
            AtaxxMove::Double(from, to) => write!(f, "{}{}", from, to),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PackedMove {
    value: u16,
}

impl PackedMove {
    pub const NONE: Self = Self::from_raw(0);
    pub const NULL: Self = Self::from_raw(1 << 12);

    #[must_use]
    const fn from_raw(value: u16) -> Self {
        Self { value }
    }

    #[must_use]
    fn pack(m: AtaxxMove) -> Self {
        match m {
            AtaxxMove::None => Self::NONE,
            AtaxxMove::Null => Self::NULL,
            AtaxxMove::Single(sq) => Self::from_raw((2 << 12) | sq.raw() as u16),
            AtaxxMove::Double(from, to) => {
                Self::from_raw((3 << 12) | ((from.raw() as u16) << 6) | (to.raw() as u16))
            }
        }
    }

    #[must_use]
    fn raw(&self) -> u16 {
        self.value
    }

    #[must_use]
    pub fn unpack(&self) -> AtaxxMove {
        match (self.value >> 12) & 0b11 {
            0 => AtaxxMove::None,
            1 => AtaxxMove::Null,
            2 => AtaxxMove::Single(self.dst_sq()),
            3 => AtaxxMove::Double(self.src_sq(), self.dst_sq()),
            _ => panic!("Invalid packed move"),
        }
    }

    #[must_use]
    fn src_sq(&self) -> Square {
        Square::from_raw(((self.value >> 6) & 0b111111) as u8)
    }

    #[must_use]
    fn dst_sq(&self) -> Square {
        Square::from_raw((self.value & 0b111111) as u8)
    }
}

#[cfg(test)]
mod tests {
    use crate::ataxx_move::{AtaxxMove, PackedMove};
    use crate::core::Square;

    #[test]
    fn pack_single() {
        let m = AtaxxMove::Single(Square::B6);
        assert_eq!(m.pack().raw(), 0b10_000000_101001);
    }

    #[test]
    fn pack_double() {
        let m = AtaxxMove::Double(Square::B5, Square::A7);
        assert_eq!(m.pack().raw(), 0b11_100001_110000);
    }

    #[test]
    fn unpack_single() {
        let m = AtaxxMove::Single(Square::B6);
        let packed = PackedMove::from_raw(0b10_000000_101001);

        assert_eq!(packed.unpack(), m);
    }

    #[test]
    fn unpack_double() {
        let m = AtaxxMove::Double(Square::B5, Square::A7);
        let packed = PackedMove::from_raw(0b11_100001_110000);

        assert_eq!(packed.unpack(), m);
    }
}
