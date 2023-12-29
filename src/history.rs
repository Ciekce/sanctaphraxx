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

const MAX_HISTORY: i16 = 16384;

const MAX_HISTORY_BONUS: i16 = 1536;
const HISTORY_SCALE: i16 = 384;
const HISTORY_OFFSET: i16 = 384;

pub struct HistoryTable {
    single: [i16; 49],
    double: [[i16; 49]; 49],
}

impl HistoryTable {
    pub fn new() -> Self {
        Self {
            single: [0; 49],
            double: [[0; 49]; 49],
        }
    }

    pub fn get_history(&self, m: AtaxxMove) -> i16 {
        assert_ne!(m, AtaxxMove::Null);
        match m {
            AtaxxMove::None => 0,
            AtaxxMove::Single(sq) => self.single[sq.idx()],
            AtaxxMove::Double(from, to) => self.double[from.idx()][to.idx()],
            _ => unreachable!()
        }
    }

    pub fn update_history(&mut self, m: AtaxxMove, bonus: i16) {
        assert_ne!(m, AtaxxMove::Null);

        let score = match m {
            AtaxxMove::None => return,
            AtaxxMove::Single(sq) => &mut self.single[sq.idx()],
            AtaxxMove::Double(from, to) => &mut self.double[from.idx()][to.idx()],
            _ => unreachable!()
        };

        *score += bonus - *score * bonus.abs() / MAX_HISTORY;
    }

    pub fn clear(&mut self) {
        self.single.fill(0);
        self.double.fill([0; 49]);
    }
}

pub fn history_bonus(depth: i32) -> i16 {
    MAX_HISTORY_BONUS.min(depth as i16 * HISTORY_SCALE - HISTORY_OFFSET)
}
