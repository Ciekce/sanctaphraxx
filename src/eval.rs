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

use crate::core::Score;
use crate::nnue;
use crate::position::Position;

#[must_use]
pub fn static_eval(pos: &Position, nnue_state: &nnue::NnueState) -> Score {
    nnue_state.evaluate(pos.side_to_move())
}

#[must_use]
pub fn static_eval_once(pos: &Position) -> Score {
    nnue::evaluate_once(pos)
}
