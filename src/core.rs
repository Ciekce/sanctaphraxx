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

use crate::bitboard::Bitboard;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Square(u8);

impl Square {
    pub const A1: Self = Self(0);
    pub const B1: Self = Self(1);
    pub const C1: Self = Self(2);
    pub const D1: Self = Self(3);
    pub const E1: Self = Self(4);
    pub const F1: Self = Self(5);
    pub const G1: Self = Self(6);
    pub const A2: Self = Self(8);
    pub const B2: Self = Self(9);
    pub const C2: Self = Self(10);
    pub const D2: Self = Self(11);
    pub const E2: Self = Self(12);
    pub const F2: Self = Self(13);
    pub const G2: Self = Self(14);
    pub const A3: Self = Self(16);
    pub const B3: Self = Self(17);
    pub const C3: Self = Self(18);
    pub const D3: Self = Self(19);
    pub const E3: Self = Self(20);
    pub const F3: Self = Self(21);
    pub const G3: Self = Self(22);
    pub const A4: Self = Self(24);
    pub const B4: Self = Self(25);
    pub const C4: Self = Self(26);
    pub const D4: Self = Self(27);
    pub const E4: Self = Self(28);
    pub const F4: Self = Self(29);
    pub const G4: Self = Self(30);
    pub const A5: Self = Self(32);
    pub const B5: Self = Self(33);
    pub const C5: Self = Self(34);
    pub const D5: Self = Self(35);
    pub const E5: Self = Self(36);
    pub const F5: Self = Self(37);
    pub const G5: Self = Self(38);
    pub const A6: Self = Self(40);
    pub const B6: Self = Self(41);
    pub const C6: Self = Self(42);
    pub const D6: Self = Self(43);
    pub const E6: Self = Self(44);
    pub const F6: Self = Self(45);
    pub const G6: Self = Self(46);
    pub const A7: Self = Self(48);
    pub const B7: Self = Self(49);
    pub const C7: Self = Self(50);
    pub const D7: Self = Self(51);
    pub const E7: Self = Self(52);
    pub const F7: Self = Self(53);
    pub const G7: Self = Self(54);

    pub const NONE: Self = Self(64);

    pub const N_SQUARES: usize = 49;

    #[must_use]
    pub const fn from_raw(value: u8) -> Self {
        debug_assert!(value == 64 || value / 8 < 7);
        debug_assert!(value == 64 || value % 8 < 7);
        Self(value)
    }

    #[must_use]
    pub const fn from_coords(rank: u32, file: u32) -> Self {
        debug_assert!(rank < 7);
        debug_assert!(file < 7);
        Self((rank * 8 + file) as u8)
    }

    #[must_use]
    pub const fn idx(&self) -> usize {
        (self.rank() * 7 + self.file()) as usize
    }

    #[must_use]
    pub const fn bit_idx(&self) -> usize {
        self.0 as usize
    }

    #[must_use]
    pub const fn bit(&self) -> Bitboard {
        Bitboard::from_raw(1 << self.bit_idx())
    }

    #[must_use]
    pub const fn rank(&self) -> u32 {
        self.0 as u32 / 8
    }

    #[must_use]
    pub const fn file(&self) -> u32 {
        self.0 as u32 % 8
    }

    #[must_use]
    pub const fn flip_horizontal(&self) -> Self {
        Self::from_coords(self.rank(), 6 - self.file())
    }

    #[must_use]
    pub const fn flip_vertical(&self) -> Self {
        Self::from_coords(6 - self.rank(), self.file())
    }
}

pub enum SquareStrError {
    WrongSize,
    InvalidFile,
    InvalidRank,
}

impl FromStr for Square {
    type Err = SquareStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(SquareStrError::WrongSize);
        }

        let mut chars = s.chars();

        let file = chars.next().unwrap();
        let rank = chars.next().unwrap();

        if !('a'..='g').contains(&file) {
            return Err(SquareStrError::InvalidFile);
        } else if !('1'..='7').contains(&rank) {
            return Err(SquareStrError::InvalidRank);
        }

        let file_idx = file as u32 - 'a' as u32;
        let rank_idx = rank as u32 - '1' as u32;

        Ok(Self::from_coords(rank_idx, file_idx))
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            char::from_u32(self.file() + 'a' as u32).unwrap(),
            char::from_u32(self.rank() + '1' as u32).unwrap()
        )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Color {
    value: u8,
}

impl Color {
    pub const RED: Self = Self::from_raw(0);
    pub const BLUE: Self = Self::from_raw(1);

    pub const NONE: Self = Self::from_raw(2);

    pub const N_COLORS: usize = 2;

    #[must_use]
    const fn from_raw(value: u8) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'x' | 'X' => Some(Self::RED),
            'o' | 'O' => Some(Self::BLUE),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_char(self) -> char {
        match self {
            Color::RED => 'x',
            Color::BLUE => 'o',
            Color::NONE => ' ',
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn flip(&self) -> Self {
        debug_assert!(*self != Self::NONE);
        Self::from_raw(self.value ^ 1)
    }

    #[must_use]
    pub fn idx(&self) -> usize {
        self.value as usize
    }
}

pub type Score = i32;

pub const SCORE_INF: Score = 32000;
pub const SCORE_MATE: Score = 31000;
pub const SCORE_WIN: Score = 30000;

pub const MAX_DEPTH: i32 = 255;

#[cfg(test)]
mod tests {
    use crate::core::Color;

    #[test]
    fn square_flip() {
        assert_eq!(Color::RED.flip(), Color::BLUE);
        assert_eq!(Color::BLUE.flip(), Color::RED);

        assert_eq!(Color::RED.flip().flip(), Color::RED);
        assert_eq!(Color::BLUE.flip().flip(), Color::BLUE);
    }
}
