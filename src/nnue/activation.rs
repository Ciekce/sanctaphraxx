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

#![allow(dead_code)]

use crate::util::simd;

pub struct ReLU;
impl ReLU {
    #[inline(always)]
    pub fn activate(v: simd::Register16) -> simd::Register16 {
        simd::max_i16(v, simd::zero16())
    }
}

pub struct ClippedReLU<const MAX: i16>;
impl<const MAX: i16> ClippedReLU<MAX> {
    #[inline(always)]
    pub fn activate(v: simd::Register16) -> simd::Register16 {
        let max = simd::set1_i16(MAX);
        simd::clamp_i16(v, simd::zero16(), max)
    }
}

pub struct SquaredClippedReLU<const MAX: i16>;
impl<const MAX: i16> SquaredClippedReLU<MAX> {
    const MAX_OVERFLOW: () = assert!((MAX as i32) * (MAX as i32) <= i16::MAX as i32);

    #[inline(always)]
    pub fn activate(v: simd::Register16) -> simd::Register16 {
        let max = simd::set1_i16(MAX);
        let clipped = simd::clamp_i16(v, simd::zero16(), max);
        simd::mul_i16(clipped, clipped)
    }
}
