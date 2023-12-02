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

impl AtaxxMove {}

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
