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
use crate::ataxx_move::AtaxxMove::*;
use crate::attacks::SINGLES;
use crate::bitboard::Bitboard;
use crate::core::{Color, Square};
use crate::hash;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
struct BoardState {
    colors: [Bitboard; 2],
    key: u64,
    last_move: AtaxxMove,
    halfmove: u16,
}

impl BoardState {
    #[must_use]
    pub fn red_occupancy(&self) -> Bitboard {
        self.colors[Color::RED.idx()]
    }

    #[must_use]
    pub fn blue_occupancy(&self) -> Bitboard {
        self.colors[Color::BLUE.idx()]
    }

    #[must_use]
    pub fn occupancy(&self) -> Bitboard {
        self.colors[0] | self.colors[1]
    }

    #[must_use]
    pub fn color_at(&self, sq: Square) -> Color {
        if self.red_occupancy().get(sq) {
            Color::RED
        } else if self.blue_occupancy().get(sq) {
            Color::BLUE
        } else {
            Color::NONE
        }
    }

    #[must_use]
    pub fn empty_squares(&self, gaps: Bitboard) -> Bitboard {
        !(self.occupancy() | gaps) & Bitboard::ALL
    }
}

impl Default for BoardState {
    #[must_use]
    fn default() -> Self {
        BoardState {
            colors: [Bitboard::EMPTY, Bitboard::EMPTY],
            key: 0,
            last_move: AtaxxMove::Null,
            halfmove: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    blue_to_move: bool,
    fullmove: u32,
    gaps: Bitboard,
    states: Vec<BoardState>,
    hashes: Vec<u64>,
}

#[derive(Debug)]
pub enum FenError {
    NotEnoughParts,
    NotEnoughRanks,
    TooManyRanks,
    NotEnoughFiles(u32),
    TooManyFiles(u32),
    InvalidChar(char),
    InvalidStm,
    InvalidHalfmove,
    InvalidFullmove,
}

impl Display for FenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FenError::NotEnoughParts => write!(f, "Incomplete FEN"),
            FenError::NotEnoughRanks => write!(f, "Not enough ranks in FEN"),
            FenError::TooManyRanks => write!(f, "Too many ranks in FEN"),
            FenError::NotEnoughFiles(rank) => write!(f, "Not enough files in rank {}", rank + 1),
            FenError::TooManyFiles(rank) => write!(f, "Too many files in rank {}", rank + 1),
            FenError::InvalidChar(c) => write!(f, "Invalid character '{}' in FEN", c),
            FenError::InvalidStm => write!(f, "Invalid side to move in FEN"),
            FenError::InvalidHalfmove => write!(f, "Invalid halfmove clock in FEN"),
            FenError::InvalidFullmove => write!(f, "Invalid fullmove number in FEN"),
        }
    }
}

#[derive(Debug)]
pub enum GameResult {
    Win(Color),
    Draw,
}

impl Position {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            blue_to_move: false,
            fullmove: 0,
            gaps: Bitboard::EMPTY,
            states: Vec::with_capacity(256),
            hashes: Vec::with_capacity(512),
        }
    }

    #[must_use]
    pub fn startpos() -> Self {
        let mut result = Self::empty();
        result.reset_to_startpos();
        result
    }

    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let mut result = Self::empty();
        result.reset_from_fen(fen)?;
        Ok(result)
    }

    pub fn reset_to_startpos(&mut self) {
        self.states.clear();
        self.states.push(BoardState {
            colors: [
                Bitboard::from_raw(0x01000000000040),
                Bitboard::from_raw(0x40000000000001),
            ],
            key: 0,
            last_move: AtaxxMove::Null,
            halfmove: 0,
        });

        self.blue_to_move = false;
        self.fullmove = 1;

        self.regen_curr_key();
    }

    pub fn reset_from_fen_parts(&mut self, parts: &[&str]) -> Result<(), FenError> {
        if parts.len() < 4 {
            return Err(FenError::NotEnoughParts);
        }

        let ranks: Vec<&str> = parts[0].split('/').collect();

        if ranks.len() < 7 {
            return Err(FenError::NotEnoughRanks);
        } else if ranks.len() > 7 {
            return Err(FenError::TooManyRanks);
        }

        let mut state = BoardState::default();
        let mut gaps = Bitboard::EMPTY;

        for (rank_idx, rank) in ranks.iter().enumerate() {
            let mut file_idx: u32 = 0;

            for c in rank.chars() {
                if file_idx >= 8 {
                    return Err(FenError::TooManyFiles(rank_idx as u32));
                }

                if let Some(empty_squares) = c.to_digit(10) {
                    file_idx += empty_squares;
                } else {
                    let sq = Square::from_coords(rank_idx as u32, file_idx).flip_vertical();

                    if let Some(color) = Color::from_char(c) {
                        state.colors[color.idx()].set(sq);
                        file_idx += 1;
                    } else if c == '-' {
                        gaps.set(sq);
                        file_idx += 1;
                    } else {
                        return Err(FenError::InvalidChar(c));
                    }
                }
            }

            if file_idx > 7 {
                return Err(FenError::TooManyFiles(rank_idx as u32));
            } else if file_idx < 7 {
                return Err(FenError::NotEnoughFiles(rank_idx as u32));
            }
        }

        if parts[1].len() != 1 {
            return Err(FenError::InvalidStm);
        }

        let blue_to_move = if let Some(stm) = Color::from_char(parts[1].chars().nth(0).unwrap()) {
            stm == Color::BLUE
        } else {
            return Err(FenError::InvalidStm);
        };

        if let Ok(halfmove) = parts[2].parse::<u16>() {
            state.halfmove = halfmove;
        } else {
            return Err(FenError::InvalidHalfmove);
        }

        let fullmove = if let Ok(fullmove) = parts[3].parse::<u32>() {
            fullmove
        } else {
            return Err(FenError::InvalidFullmove);
        };

        self.blue_to_move = blue_to_move;
        self.fullmove = fullmove;
        self.gaps = gaps;

        self.states.clear();
        self.states.push(state);

        self.hashes.clear();

        self.regen_curr_key();

        Ok(())
    }

    pub fn reset_from_fen(&mut self, fen: &str) -> Result<(), FenError> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        self.reset_from_fen_parts(parts.as_slice())
    }

    fn regen_curr_key(&mut self) {
        let blue_to_move = self.blue_to_move;
        let state = self.curr_state_mut();

        state.key = 0;

        for red_piece in state.red_occupancy() {
            state.key ^= hash::color_square_key(Color::RED, red_piece);
        }

        for blue_piece in state.blue_occupancy() {
            state.key ^= hash::color_square_key(Color::BLUE, blue_piece);
        }

        if blue_to_move {
            state.key ^= hash::stm_key();
        }
    }

    #[must_use]
    fn curr_state(&self) -> &BoardState {
        self.states.last().unwrap()
    }

    #[must_use]
    fn curr_state_mut(&mut self) -> &mut BoardState {
        self.states.last_mut().unwrap()
    }

    #[must_use]
    pub fn game_over(&self) -> bool {
        let state = self.curr_state();
        state.red_occupancy().is_empty()
            || state.blue_occupancy().is_empty()
            || state.occupancy() == Bitboard::ALL
            || state.halfmove >= 100
            || (state.occupancy().expand().expand() & state.empty_squares(self.gaps)).is_empty()
    }

    #[must_use]
    pub fn result(&self) -> GameResult {
        let state = self.curr_state();

        let red_count = state.red_occupancy().popcount();
        let blue_count = state.blue_occupancy().popcount();

        return match red_count.cmp(&blue_count) {
            Ordering::Less => GameResult::Win(Color::BLUE),
            Ordering::Equal => GameResult::Draw,
            Ordering::Greater => GameResult::Win(Color::RED),
        };
    }

    #[must_use]
    pub fn is_legal(&self, m: AtaxxMove) -> bool {
        assert_ne!(m, AtaxxMove::None);

        let state = self.curr_state();

        let ours = self.color_occupancy(self.side_to_move());
        if ours.is_empty() {
            return false;
        }

        let empty = state.empty_squares(self.gaps);

        match m {
            Null => (ours.expand().expand() & empty).is_empty(),
            Single(sq) => (ours.expand() & empty).get(sq),
            Double(from, to) => {
                let singles = ours.expand();
                ours.get(from) && (singles.expand() & !singles & empty).get(to)
            }
            _ => unreachable!(),
        }
    }

    pub fn apply_move<const HISTORY: bool, const UPDATE_KEY: bool>(&mut self, m: AtaxxMove) {
        debug_assert!(m != AtaxxMove::None);

        let us = self.side_to_move();
        let them = us.flip();

        self.blue_to_move = !self.blue_to_move;

        let mut new_state = self.curr_state().clone();
        self.curr_state_mut().last_move = m;

        if UPDATE_KEY {
            self.hashes.push(new_state.key);
            new_state.key ^= hash::stm_key();
        }

        if us == Color::BLUE {
            self.fullmove += 1;
        }

        let (from, to) = match m {
            Single(to) => {
                new_state.halfmove = 0;
                (to, to)
            }
            Double(from, to) => {
                new_state.halfmove += 1;
                (from, to)
            }
            _ => {
                self.states.push(new_state);
                return;
            }
        };

        let mut ours = new_state.colors[us.idx()];
        let mut theirs = new_state.colors[them.idx()];

        ours ^= from.bit() | to.bit();

        let captured = SINGLES[to.bit_idx()] & theirs;

        ours ^= captured;
        theirs ^= captured;

        new_state.colors[us.idx()] = ours;
        new_state.colors[them.idx()] = theirs;

        if UPDATE_KEY {
            new_state.key ^= hash::color_square_key(us, to);

            if from != to {
                new_state.key ^= hash::color_square_key(us, from);
            }

            for sq in captured {
                new_state.key ^= hash::color_square_key(us, sq);
                new_state.key ^= hash::color_square_key(them, sq);
            }
        }

        if HISTORY {
            self.states.push(new_state);
        } else {
            *self.curr_state_mut() = new_state;
        }
    }

    pub fn pop_move<const UPDATE_KEY: bool>(&mut self) {
        debug_assert!(self.states.len() > 1);

        let m = self.states.pop().unwrap().last_move;

        if UPDATE_KEY {
            self.hashes.pop();
        }

        self.blue_to_move = !self.blue_to_move;

        if m != AtaxxMove::Null && self.blue_to_move {
            self.fullmove -= 1;
        }
    }

    #[must_use]
    pub fn side_to_move(&self) -> Color {
        if self.blue_to_move {
            Color::BLUE
        } else {
            Color::RED
        }
    }

    #[must_use]
    pub fn occupancy(&self) -> Bitboard {
        self.curr_state().occupancy()
    }

    #[must_use]
    pub fn empty_squares(&self) -> Bitboard {
        self.curr_state().empty_squares(self.gaps)
    }

    #[must_use]
    pub fn red_occupancy(&self) -> Bitboard {
        self.curr_state().red_occupancy()
    }

    #[must_use]
    pub fn blue_occupancy(&self) -> Bitboard {
        self.curr_state().blue_occupancy()
    }

    #[must_use]
    pub fn color_occupancy(&self, color: Color) -> Bitboard {
        self.curr_state().colors[color.idx()]
    }

    #[must_use]
    pub fn color_at(&self, sq: Square) -> Color {
        self.curr_state().color_at(sq)
    }

    #[must_use]
    pub fn gap_at(&self, sq: Square) -> bool {
        self.gaps.get(sq)
    }

    #[must_use]
    pub fn key(&self) -> u64 {
        self.curr_state().key
    }

    #[must_use]
    pub fn to_fen(&self) -> String {
        let state = self.curr_state();

        let mut fen = String::new();

        for rank in (0u32..7).rev() {
            let mut file: u32 = 0;

            while file < 7 {
                let sq = Square::from_coords(rank, file);

                match state.color_at(sq) {
                    Color::RED => fen.push('x'),
                    Color::BLUE => fen.push('o'),
                    Color::NONE => {
                        if self.gap_at(sq) {
                            fen.push('-');
                        } else {
                            let mut empty_squares: u32 = 1;

                            while file < 6
                                && state.color_at(Square::from_coords(rank, file + 1))
                                    == Color::NONE
                            {
                                file += 1;
                                empty_squares += 1;
                            }

                            fen += empty_squares.to_string().as_str();
                        }
                    }
                    _ => unreachable!(),
                }

                file += 1;
            }

            if rank > 0 {
                fen.push('/');
            }
        }

        fen + format!(
            " {} {} {}",
            self.side_to_move().to_char(),
            state.halfmove,
            self.fullmove
        )
        .as_str()
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for rank in (0u32..7).rev() {
            writeln!(f, " +---+---+---+---+---+---+---+")?;

            for file in 0u32..7 {
                let sq = Square::from_coords(rank, file);
                write!(
                    f,
                    " | {}",
                    if self.gap_at(sq) {
                        '-'
                    } else {
                        self.color_at(sq).to_char()
                    }
                )?;
            }

            writeln!(f, " | {}", rank + 1)?;
        }

        writeln!(f, " +---+---+---+---+---+---+---+")?;
        writeln!(f, "   a   b   c   d   e   f   g")?;
        writeln!(f)?;

        write!(
            f,
            "{} to move",
            if self.side_to_move() == Color::RED {
                "Red"
            } else {
                "Blue"
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::ataxx_move::AtaxxMove;
    use crate::core::Square;
    use crate::position::Position;

    #[test]
    fn noncapture_single_key() {
        let mut pos = Position::startpos();
        pos.apply_move::<false, true>(AtaxxMove::Single(Square::B6));

        let incr_key = pos.key();

        pos.regen_curr_key();
        let regen_key = pos.key();

        assert_eq!(incr_key, regen_key);
    }

    #[test]
    fn noncapture_double_key() {
        let mut pos = Position::startpos();
        pos.apply_move::<false, true>(AtaxxMove::Double(Square::A7, Square::C5));

        let incr_key = pos.key();

        pos.regen_curr_key();
        let regen_key = pos.key();

        assert_eq!(incr_key, regen_key);
    }

    #[test]
    fn capture_single_key() {
        let mut pos = Position::from_fen("x5o/2o4/7/7/7/7/o5x x 0 1").unwrap();
        pos.apply_move::<false, true>(AtaxxMove::Single(Square::B6));

        let incr_key = pos.key();

        pos.regen_curr_key();
        let regen_key = pos.key();

        assert_eq!(incr_key, regen_key);
    }

    #[test]
    fn capture_double_key() {
        let mut pos = Position::from_fen("x5o/2o4/7/7/7/7/o5x x 0 1").unwrap();
        pos.apply_move::<false, true>(AtaxxMove::Double(Square::A7, Square::C5));

        let incr_key = pos.key();

        pos.regen_curr_key();
        let regen_key = pos.key();

        assert_eq!(incr_key, regen_key);
    }
}
