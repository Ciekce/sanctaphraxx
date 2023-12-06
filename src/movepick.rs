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
use crate::movegen::{fill_move_list, MoveList};
use crate::position::Position;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum MovepickerStage {
    Start,
    TtMove,
    GenerateMoves,
    End,
}

impl MovepickerStage {
    fn next(&self) -> Self {
        match self {
            Self::Start => Self::TtMove,
            Self::TtMove => Self::GenerateMoves,
            Self::GenerateMoves => Self::End,
            Self::End => Self::End,
        }
    }
}

pub struct Movepicker {
    stage: MovepickerStage,
    tt_move: AtaxxMove,
    generated_moves: MoveList,
    idx: usize,
}

impl Movepicker {
    #[must_use]
    pub fn new(tt_move: AtaxxMove) -> Self {
        Self {
            stage: MovepickerStage::Start,
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
                        if self.tt_move != AtaxxMove::None && pos.is_legal(self.tt_move) {
                            return Some(self.tt_move);
                        }
                    }
                    MovepickerStage::GenerateMoves => {
                        fill_move_list(&mut self.generated_moves, pos)
                    }
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

#[cfg(test)]
mod tests {
    use crate::ataxx_move::AtaxxMove;
    use crate::core::Square;
    use crate::movegen::{fill_move_list, MoveList};
    use crate::movepick::Movepicker;
    use crate::position::Position;

    #[test]
    fn tt_move_first() {
        let pos = Position::startpos();

        let mut moves = MoveList::new();
        fill_move_list(&mut moves, &pos);

        // arbitrary
        let tt_move = moves[3];
        let mut movepicker = Movepicker::new(tt_move);

        assert_eq!(tt_move, movepicker.next(&pos).unwrap());
    }

    #[test]
    fn tt_move_unduplicated() {
        let pos = Position::startpos();

        let mut moves = MoveList::new();
        fill_move_list(&mut moves, &pos);

        let tt_move = moves[3];
        let mut movepicker = Movepicker::new(tt_move);

        movepicker.next(&pos);

        while let Some(m) = movepicker.next(&pos) {
            assert_ne!(m, tt_move);
        }
    }

    #[test]
    fn no_move_missing() {
        let pos = Position::startpos();

        let mut moves = MoveList::new();
        fill_move_list(&mut moves, &pos);

        let tt_move = moves[3];
        let mut movepicker = Movepicker::new(tt_move);

        let mut move_count = 0usize;

        while movepicker.next(&pos).is_some() {
            move_count += 1;
        }

        assert_eq!(move_count, moves.len());
    }

    #[test]
    fn no_move_missing_no_tt_move() {
        let pos = Position::startpos();

        let mut moves = MoveList::new();
        fill_move_list(&mut moves, &pos);

        let mut movepicker = Movepicker::new(AtaxxMove::None);

        let mut move_count = 0usize;

        while movepicker.next(&pos).is_some() {
            move_count += 1;
        }

        assert_eq!(move_count, moves.len());
    }

    #[test]
    fn illegal_tt_move() {
        let pos = Position::startpos();

        let mut moves = MoveList::new();
        fill_move_list(&mut moves, &pos);

        let tt_move = AtaxxMove::Single(Square::B1);
        let mut movepicker = Movepicker::new(tt_move);

        while let Some(m) = movepicker.next(&pos) {
            assert_ne!(m, tt_move);
        }
    }
}
