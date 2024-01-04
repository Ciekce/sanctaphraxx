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

use crate::nnue::activation;
use crate::util::simd;

pub const L1_Q: i32 = 255;
pub const OUTPUT_Q: i32 = 64;

pub const INPUT_SIZE: usize = 147;
pub const L1_SIZE: usize = 64;

pub type Activation = activation::ClippedReLU<{ L1_Q as i16 }>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(align(64))]
pub struct Align64<T>(pub T);

#[repr(C)]
pub struct Layer<T, const INPUTS: usize, const WEIGHTS: usize, const OUTPUTS: usize> {
    pub weights: Align64<[T; WEIGHTS]>,
    pub biases: Align64<[T; OUTPUTS]>,
}

impl<T, const INPUTS: usize, const WEIGHTS: usize, const OUTPUTS: usize>
    Layer<T, INPUTS, WEIGHTS, OUTPUTS>
{
    pub fn weight_ptr(&self, feature: usize, idx: usize) -> *const simd::Register16 {
        assert!(feature * INPUTS + idx < WEIGHTS);
        assert_eq!(idx % simd::CHUNK_SIZE_I16, 0);

        unsafe { self.weights.0.as_ptr().add(feature * INPUTS + idx).cast() }
    }
}

#[repr(C)]
pub struct Network {
    pub feature_transformer: Layer<i16, INPUT_SIZE, { INPUT_SIZE * L1_SIZE }, L1_SIZE>,
    pub l1: Layer<i16, L1_SIZE, { L1_SIZE * 2 }, 1>,
}

pub const NETWORK: Network = unsafe { std::mem::transmute(*include_bytes!("random.nnue")) };
