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

use crate::bitboard::Bitboard;
use crate::c_for;
use crate::core::Square;

#[must_use]
const fn generate_singles(board: Bitboard) -> Bitboard {
    board.expand().and(board.inverse())
}

pub const SINGLES: [Bitboard; 64] = {
    let mut result = [Bitboard::EMPTY; 64];

    c_for!(let mut rank = 0u32; rank < 7; rank += 1; {
        c_for!(let mut file = 0u32; file < 7; file += 1; {
            let sq = Square::from_coords(rank, file);
            result[sq.bit_idx()] = generate_singles(sq.bit());
        });
    });

    result
};

pub const DOUBLES: [Bitboard; 64] = {
    let mut result = [Bitboard::EMPTY; 64];

    c_for!(let mut rank = 0u32; rank < 7; rank += 1; {
        c_for!(let mut file = 0u32; file < 7; file += 1; {
            let sq = Square::from_coords(rank, file);
            let idx = sq.bit_idx();
            result[idx] = SINGLES[idx].expand().and(sq.bit().expand().inverse());
        });
    });

    result
};
