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
use crate::util::rng;

const COLOR_SQUARE_SIZE: usize = Color::N_COLORS * Square::N_SQUARES;
const STM_SIZE: usize = 1;

const TOTAL_SIZE: usize = COLOR_SQUARE_SIZE + STM_SIZE;

const COLOR_SQUARE_OFFSET: usize = 0;
const STM_OFFSET: usize = COLOR_SQUARE_OFFSET + COLOR_SQUARE_SIZE;

const HASHES: [u64; TOTAL_SIZE] = rng::fill_u64_array(0x22ff7d8af027681b);

#[must_use]
pub fn color_square_key(c: Color, sq: Square) -> u64 {
    debug_assert!(c != Color::NONE);
    debug_assert!(sq != Square::NONE);

    HASHES[COLOR_SQUARE_OFFSET + sq.idx() * Color::N_COLORS + c.idx()]
}

#[must_use]
pub fn stm_key() -> u64 {
    HASHES[STM_OFFSET]
}
