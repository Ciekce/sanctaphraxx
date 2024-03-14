/*
 * Sanctaphraxx, a UAI Ataxx engine
 * Copyright (C) 2024 Ciekce
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

use crate::core::*;
use std::fmt::{Display, Formatter};
use std::ops::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Bitboard {
    value: u64,
}

#[allow(unused, clippy::unreadable_literal)]
impl Bitboard {
    pub const RANK_1: Self = Self::from_raw(0x0000000000007f);
    pub const RANK_2: Self = Self::from_raw(0x00000000007f00);
    pub const RANK_3: Self = Self::from_raw(0x000000007f0000);
    pub const RANK_4: Self = Self::from_raw(0x0000007f000000);
    pub const RANK_5: Self = Self::from_raw(0x00007f00000000);
    pub const RANK_6: Self = Self::from_raw(0x007f0000000000);
    pub const RANK_7: Self = Self::from_raw(0x7f000000000000);

    pub const FILE_A: Self = Self::from_raw(0x01010101010101);
    pub const FILE_B: Self = Self::from_raw(0x02020202020202);
    pub const FILE_C: Self = Self::from_raw(0x04040404040404);
    pub const FILE_D: Self = Self::from_raw(0x08080808080808);
    pub const FILE_E: Self = Self::from_raw(0x10101010101010);
    pub const FILE_F: Self = Self::from_raw(0x20202020202020);
    pub const FILE_G: Self = Self::from_raw(0x40404040404040);

    pub const ALL: Self = Self::from_raw(0x7f7f7f7f7f7f7f);
    pub const EMPTY: Self = Self::from_raw(0);

    #[must_use]
    pub const fn from_raw(value: u64) -> Self {
        Self { value }
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.value
    }

    #[must_use]
    pub const fn popcount(self) -> u32 {
        self.value.count_ones()
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.value == 0
    }

    #[must_use]
    pub const fn contains_multiple(self) -> bool {
        self.value != self.value & (self.value - 1)
    }

    #[must_use]
    pub const fn get(self, sq: Square) -> bool {
        !self.and(sq.bit()).is_empty()
    }

    pub fn set(&mut self, sq: Square) {
        *self |= sq.bit();
    }

    pub fn clear(&mut self, sq: Square) {
        *self &= !sq.bit();
    }

    pub fn flip(&mut self, sq: Square) {
        *self ^= sq.bit();
    }

    pub fn set_to(&mut self, sq: Square, value: bool) {
        if value {
            self.set(sq);
        } else {
            self.clear(sq);
        }
    }

    #[must_use]
    pub const fn and(self, rhs: Bitboard) -> Self {
        Self {
            value: self.value & rhs.value,
        }
    }

    #[must_use]
    pub const fn or(self, rhs: Bitboard) -> Self {
        Self {
            value: self.value | rhs.value,
        }
    }

    #[must_use]
    pub const fn xor(self, rhs: Bitboard) -> Self {
        Self {
            value: self.value ^ rhs.value,
        }
    }

    #[must_use]
    pub const fn inverse(self) -> Self {
        Self { value: !self.value }
    }

    #[must_use]
    pub const fn bitshift_left(self, rhs: u32) -> Self {
        Self {
            value: self.value << rhs,
        }
    }

    #[must_use]
    pub const fn bitshift_right(self, rhs: u32) -> Self {
        Self {
            value: self.value >> rhs,
        }
    }

    #[must_use]
    pub const fn shift_up(self) -> Self {
        const MASK: Bitboard = Bitboard::ALL;
        Self {
            value: self.value << 8,
        }
        .and(MASK)
    }

    #[must_use]
    pub const fn shift_down(self) -> Self {
        Self {
            value: self.value >> 8,
        }
    }

    #[must_use]
    pub const fn shift_left(self) -> Self {
        // normally in chess you would need to also mask off the H file
        // but there is no H file in ataxx
        const MASK: Bitboard = Bitboard::ALL;
        Self {
            value: self.value << 1,
        }
        .and(MASK)
    }

    #[must_use]
    pub const fn shift_right(self) -> Self {
        // see above
        // can't wrap anything to A if there is no H
        const MASK: Bitboard = Bitboard::ALL;
        Self {
            value: self.value >> 1,
        }
        .and(MASK)
    }

    #[must_use]
    pub const fn shift_up_left(self) -> Self {
        const MASK: Bitboard = Bitboard::ALL;
        Self {
            value: self.value << 7,
        }
        .and(MASK)
    }

    #[must_use]
    pub const fn shift_up_right(self) -> Self {
        const MASK: Bitboard = Bitboard::ALL;
        Self {
            value: self.value << 9,
        }
        .and(MASK)
    }

    #[must_use]
    pub const fn shift_down_left(self) -> Self {
        const MASK: Bitboard = Bitboard::ALL;
        Self {
            value: self.value >> 9,
        }
        .and(MASK)
    }

    #[must_use]
    pub const fn shift_down_right(self) -> Self {
        const MASK: Bitboard = Bitboard::ALL;
        Self {
            value: self.value >> 7,
        }
        .and(MASK)
    }

    #[must_use]
    pub const fn expand(self) -> Self {
        let board = self.or(self.shift_up().or(self.shift_down()));
        board
            .or(board.shift_left().or(board.shift_right()))
            .and(Bitboard::ALL)
    }

    #[must_use]
    pub const fn lowest_square(self) -> Square {
        Square::from_raw(self.value.trailing_zeros() as u8)
    }

    pub fn pop_lowest_square(&mut self) -> Square {
        let square = self.lowest_square();
        self.value &= self.value - 1;
        square
    }
}

impl IntoIterator for Bitboard {
    type Item = Square;
    type IntoIter = Biterator;

    #[must_use]
    fn into_iter(self) -> Self::IntoIter {
        Biterator { board: self }
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    #[must_use]
    fn bitand(self, rhs: Self) -> Self::Output {
        self.and(rhs)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.value &= rhs.value;
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    #[must_use]
    fn bitor(self, rhs: Self) -> Self::Output {
        self.or(rhs)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.value |= rhs.value;
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    #[must_use]
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.xor(rhs)
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.value ^= rhs.value;
    }
}

impl Not for Bitboard {
    type Output = Self;

    #[must_use]
    fn not(self) -> Self::Output {
        self.inverse()
    }
}

impl Shr<u32> for Bitboard {
    type Output = Self;

    #[must_use]
    fn shr(self, rhs: u32) -> Self::Output {
        self.bitshift_right(rhs)
    }
}

impl ShrAssign<u32> for Bitboard {
    fn shr_assign(&mut self, rhs: u32) {
        self.value >>= rhs;
    }
}

impl Shl<u32> for Bitboard {
    type Output = Self;

    #[must_use]
    fn shl(self, rhs: u32) -> Self::Output {
        self.bitshift_left(rhs)
    }
}

impl ShlAssign<u32> for Bitboard {
    fn shl_assign(&mut self, rhs: u32) {
        self.value <<= rhs;
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for rank in (0u32..7).rev() {
            for file in 0u32..7 {
                if file > 0 {
                    write!(f, " ")?;
                }

                write!(
                    f,
                    "{}",
                    if self.get(Square::from_coords(rank, file)) {
                        '1'
                    } else {
                        '0'
                    }
                )?;
            }

            if rank > 0 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

pub struct Biterator {
    board: Bitboard,
}

impl Iterator for Biterator {
    type Item = Square;

    #[must_use]
    fn next(&mut self) -> Option<Self::Item> {
        if self.board.is_empty() {
            None
        } else {
            Some(self.board.pop_lowest_square())
        }
    }
}
