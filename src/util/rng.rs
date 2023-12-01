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

use crate::util::misc::c_for;

#[derive(Debug, Copy, Clone)]
pub struct Jsf64Rng {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

impl Jsf64Rng {
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        let mut result = Self {
            a: 0xF1EA5EED,
            b: seed,
            c: seed,
            d: seed,
        };

        c_for!(let mut i = 0; i < 20; i += 1; {
            result.next_u64();
        });

        result
    }

    pub const fn next_u64(&mut self) -> u64 {
        let e = self.a.wrapping_sub(self.b.rotate_left(7));
        self.a = self.b ^ self.c.rotate_left(13);
        self.b = self.c.wrapping_add(self.d.rotate_left(37));
        self.c = self.d.wrapping_add(e);
        self.d = e.wrapping_add(self.a);
        self.d
    }
}
