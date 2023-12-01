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

use crate::bitboard::Bitboard;
use crate::core::{Color, Square};
use crate::hash;

struct BoardState {
    colors: [Bitboard; 2],
    gaps: Bitboard,
    key: u64,
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
    pub fn gap_at(&self, sq: Square) -> bool {
        self.gaps.get(sq)
    }
}

impl Default for BoardState {
    #[must_use]
    fn default() -> Self {
        BoardState {
            colors: [Bitboard::EMPTY, Bitboard::EMPTY],
            gaps: Bitboard::EMPTY,
            key: 0,
            halfmove: 0,
        }
    }
}

pub struct Position {
    blue_to_move: bool,
    fullmove: u32,
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

impl Position {
    #[must_use]
    fn empty() -> Self {
        Self {
            blue_to_move: false,
            fullmove: 0,
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
            gaps: Bitboard::EMPTY,
            key: 0,
            halfmove: 0,
        });

        self.blue_to_move = false;
        self.fullmove = 1;

        self.regen_key();
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

        for (rank_idx, rank) in ranks.iter().enumerate() {
            let mut file_idx: u32 = 0;

            for c in rank.chars() {
                if file_idx >= 8 {
                    return Err(FenError::TooManyFiles(rank_idx as u32));
                }

                if let Some(empty_squares) = c.to_digit(10) {
                    file_idx += empty_squares;
                } else if let Some(color) = Color::from_char(c) {
                    state.colors[color.idx()]
                        .set(Square::from_coords(rank_idx as u32, file_idx).flip_vertical());
                    file_idx += 1;
                } else {
                    return Err(FenError::InvalidChar(c));
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

        self.states.clear();
        self.states.push(state);

        self.hashes.clear();

        self.regen_key();

        Ok(())
    }

    pub fn reset_from_fen(&mut self, fen: &str) -> Result<(), FenError> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        self.reset_from_fen_parts(parts.as_slice())
    }

    fn regen_key(&mut self) {
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
    pub fn to_move(&self) -> Color {
        if self.blue_to_move {
            Color::BLUE
        } else {
            Color::RED
        }
    }

    #[must_use]
    pub fn color_at(&self, sq: Square) -> Color {
        self.curr_state().color_at(sq)
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
                        if state.gap_at(sq) {
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
            self.to_move().to_char(),
            state.halfmove,
            self.fullmove
        )
        .as_str()
    }
}
