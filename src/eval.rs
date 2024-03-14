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

use crate::core::{Score, SCORE_WIN};
use crate::nnue;
use crate::position::Position;

#[must_use]
pub fn static_eval(pos: &Position, nnue_state: &nnue::NnueState) -> Score {
    let eval = nnue_state.evaluate(pos.side_to_move());
    eval.clamp(-SCORE_WIN + 1, SCORE_WIN - 1)
}

#[must_use]
pub fn static_eval_once(pos: &Position) -> Score {
    let eval = nnue::evaluate_once(pos);
    eval.clamp(-SCORE_WIN + 1, SCORE_WIN - 1)
}
