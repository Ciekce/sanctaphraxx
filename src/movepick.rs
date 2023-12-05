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
use crate::movegen::{fill_move_list, fill_scored_move_list, MoveList, ScoredMoveList};
use crate::position::Position;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum MovepickerStage {
    Start,
    TtMove,
    GenerateMoves,
    Moves,
    End,
}

impl MovepickerStage {
    fn next(&self) -> Self {
        match self {
            MovepickerStage::Start => MovepickerStage::TtMove,
            MovepickerStage::TtMove => MovepickerStage::GenerateMoves,
            MovepickerStage::GenerateMoves => MovepickerStage::Moves,
            MovepickerStage::Moves => MovepickerStage::End,
            MovepickerStage::End => MovepickerStage::End
        }
    }
}

pub struct Movepicker {
    stage: MovepickerStage,
    tt_move: AtaxxMove,
    generated_moves: MoveList,
    idx: usize
}

impl Movepicker {
    pub fn new(tt_move: AtaxxMove) -> Self {
        Self {
            stage: MovepickerStage::TtMove,
            tt_move,
            generated_moves: MoveList::new(),
            idx: 0,
        }
    }

    pub fn next(&mut self, pos: &Position) -> Option<AtaxxMove> {
        loop {
            while self.idx == self.generated_moves.len() {
                self.stage = self.stage.next();
                match self.stage {
                    MovepickerStage::TtMove => {
                        if self.tt_move != AtaxxMove::None {
                            return Some(self.tt_move);
                        }
                    },
                    MovepickerStage::GenerateMoves => {
                        fill_move_list(&mut self.generated_moves, pos);
                    },
                    MovepickerStage::End => return None,
                    _ => {}
                }
            }

            let m = self.generated_moves[self.idx];
            self.idx += 1;

            if m != self.tt_move {
                return Some(m);
            }
        }
    }
}
