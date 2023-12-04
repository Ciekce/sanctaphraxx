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

use crate::ataxx_move::{AtaxxMove, PackedMove};
use crate::core::{Score, MAX_DEPTH, SCORE_INF};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TtEntryFlag {
    None,
    Exact,
    Alpha,
    Beta,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TtEntry {
    pub key: u16,
    pub m: PackedMove,
    pub score: i16,
    pub depth: u8,
    pub flag: TtEntryFlag,
}

const _: () = assert!(std::mem::size_of::<TtEntry>() == 8);

impl Default for TtEntry {
    #[must_use]
    fn default() -> Self {
        Self {
            key: 0,
            score: 0,
            m: PackedMove::NONE,
            depth: 0,
            flag: TtEntryFlag::None,
        }
    }
}

pub struct TTable {
    table: Vec<TtEntry>,
}

impl TTable {
    pub const DEFAULT_SIZE_MB: usize = 64;

    pub const MIN_SIZE_MB: usize = 1;
    pub const MAX_SIZE_MB: usize = 131072;

    #[must_use]
    pub fn new() -> Self {
        let mut result = Self { table: Vec::new() };

        result.resize(Self::DEFAULT_SIZE_MB);

        result
    }

    pub fn resize(&mut self, capacity: usize) {
        let bytes = capacity * 1024 * 1024;
        let new_size = bytes / std::mem::size_of::<TtEntry>();

        self.table.clear();
        self.table.shrink_to_fit();

        self.table.resize_with(new_size, TtEntry::default);
    }

    pub fn clear(&mut self) {
        self.table.fill(TtEntry::default());
    }

    #[must_use]
    pub fn probe(&self, key: u64) -> Option<TtEntry> {
        let entry = self.table[self.index(key)];
        if entry.flag == TtEntryFlag::None || entry.key != Self::pack_key(key) {
            None
        } else {
            Some(entry)
        }
    }

    pub fn store(&mut self, key: u64, m: AtaxxMove, score: Score, depth: i32, flag: TtEntryFlag) {
        debug_assert!(score.abs() < SCORE_INF);
        debug_assert!((0..=MAX_DEPTH).contains(&depth));

        let idx = self.index(key);
        self.table[idx] = TtEntry {
            key: Self::pack_key(key),
            m: m.pack(),
            score: score as i16,
            depth: depth as u8,
            flag,
        };
    }

    #[must_use]
    fn index(&self, key: u64) -> usize {
        (((key as u128) * (self.table.len() as u128)) >> 64) as usize
    }

    #[must_use]
    fn pack_key(key: u64) -> u16 {
        (key & 0xFFFF) as u16
    }
}
