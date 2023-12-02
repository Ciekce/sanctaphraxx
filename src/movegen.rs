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

use crate::ataxx_move::AtaxxMove;
use crate::attacks::DOUBLES;
use crate::position::Position;

pub type MoveList = arrayvec::ArrayVec<AtaxxMove, 128>;

pub fn generate_moves(dst: &mut MoveList, pos: &Position) {
    if pos.game_over() {
        return;
    }

    let ours = pos.color_occupancy(pos.side_to_move());
    let empty = pos.empty_squares();

    let singles = ours.expand() & empty;

    for to in singles {
        dst.push(AtaxxMove::Single(to));
    }

    for from in ours {
        let attacks = DOUBLES[from.bit_idx()] & empty;
        for to in attacks {
            dst.push(AtaxxMove::Double(from, to));
        }
    }

    if dst.is_empty() {
        dst.push(AtaxxMove::Null);
    }
}
