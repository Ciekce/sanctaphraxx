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

use crate::c_for;
use crate::core::*;
use crate::next_jsf64;
use crate::util::rng::Jsf64Rng;

const COLOR_SQUARE_SIZE: usize = Color::N_COLORS * Square::N_SQUARES;
const STM_SIZE: usize = 1;

const TOTAL_SIZE: usize = COLOR_SQUARE_SIZE + STM_SIZE;

const COLOR_SQUARE_OFFSET: usize = 0;
const STM_OFFSET: usize = COLOR_SQUARE_OFFSET + COLOR_SQUARE_SIZE;

const HASHES: [u64; TOTAL_SIZE] = {
    let mut rng = Jsf64Rng::new(1234);
    let mut hashes = [0u64; TOTAL_SIZE];

    c_for!(let mut i: usize = 0; i < TOTAL_SIZE; i += 1; {
        hashes[i] = next_jsf64!(rng);
    });

    hashes
};

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
