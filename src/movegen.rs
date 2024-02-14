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

use crate::ataxx_move::AtaxxMove;
use crate::attacks::DOUBLES;
use crate::position::Position;

pub type MoveList = arrayvec::ArrayVec<AtaxxMove, 200>;
pub type ScoredMoveList = arrayvec::ArrayVec<(AtaxxMove, i32), 200>;

fn generate_moves<Callback>(pos: &Position, mut callback: Callback)
where
    Callback: FnMut(AtaxxMove),
{
    if pos.game_over() {
        return;
    }

    let mut must_pass = true;

    let ours = pos.color_occupancy(pos.side_to_move());
    let empty = pos.empty_squares();

    let singles = ours.expand() & empty;

    for to in singles {
        callback(AtaxxMove::Single(to));
        must_pass = false;
    }

    for from in ours {
        let attacks = DOUBLES[from.bit_idx()] & empty;
        for to in attacks {
            callback(AtaxxMove::Double(from, to));
            must_pass = false;
        }
    }

    if must_pass {
        callback(AtaxxMove::Null);
    }
}

pub fn fill_move_list(moves: &mut MoveList, pos: &Position) {
    generate_moves(pos, |m| moves.push(m));
}

pub fn fill_scored_move_list(moves: &mut ScoredMoveList, pos: &Position) {
    generate_moves(pos, |m| moves.push((m, 0)));
}
