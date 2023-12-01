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

macro_rules! next_jsf64 {
    ($s:ident) => {{
        let e = $s.a.wrapping_sub($s.b.rotate_left(7));
        $s.a = $s.b ^ $s.c.rotate_left(13);
        $s.b = $s.c.wrapping_add($s.d.rotate_left(37));
        $s.c = $s.d.wrapping_add(e);
        $s.d = e.wrapping_add($s.a);
        $s.d
    }};
}

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
            next_jsf64!(result)
        });

        result
    }

    pub fn next_u64(&mut self) -> u64 {
        next_jsf64!(self)
    }
}

pub const fn fill_u64_array<const SIZE: usize>(seed: u64) -> [u64; SIZE] {
    let mut rng = Jsf64Rng::new(seed);
    let mut result = [0u64; SIZE];

    c_for!(let mut i: usize = 0; i < SIZE; i += 1; {
        result[i] = next_jsf64!(rng);
    });

    result
}
