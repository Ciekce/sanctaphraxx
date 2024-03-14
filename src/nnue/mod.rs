/*
 * Sanctaphraxx, a UAI Ataxx engine
 * Copyright (C) 2024 Ciekce
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

use crate::core::*;
use crate::nnue::network::*;
use crate::position::Position;
use crate::util::simd;

mod activation;
mod network;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(align(64))]
pub struct Align64<T>(pub T);

#[derive(Debug, Copy, Clone)]
struct Accumulator {
    values: Align64<[i16; L1_SIZE]>,
}

impl Accumulator {
    fn value_ptr(&self, idx: usize) -> *const simd::Register16 {
        debug_assert!(idx < L1_SIZE);
        unsafe { self.values.0.as_ptr().add(idx).cast() }
    }

    fn value_ptr_mut(&mut self, idx: usize) -> *mut simd::Register16 {
        debug_assert!(idx < L1_SIZE);
        unsafe { self.values.0.as_mut_ptr().add(idx).cast() }
    }

    fn activate_feature(&mut self, feature: usize) {
        for i in (0..L1_SIZE).step_by(simd::CHUNK_SIZE_I16) {
            let ptr = self.value_ptr_mut(i);

            let values = unsafe { simd::load16(ptr) };
            let weights =
                unsafe { simd::load16(NETWORK.feature_transformer.weight_ptr(feature, i)) };

            let new = simd::add_i16(values, weights);

            unsafe {
                simd::store16(ptr, new);
            }
        }
    }

    fn deactivate_feature(&mut self, feature: usize) {
        for i in (0..L1_SIZE).step_by(simd::CHUNK_SIZE_I16) {
            let ptr = self.value_ptr_mut(i);

            let values = unsafe { simd::load16(ptr) };
            let weights =
                unsafe { simd::load16(NETWORK.feature_transformer.weight_ptr(feature, i)) };

            let new = simd::sub_i16(values, weights);

            unsafe {
                simd::store16(ptr, new);
            }
        }
    }

    fn move_feature(&mut self, src_feature: usize, dst_feature: usize) {
        for i in (0..L1_SIZE).step_by(simd::CHUNK_SIZE_I16) {
            let ptr = self.value_ptr_mut(i);

            let values = unsafe { simd::load16(ptr) };

            let src_weights =
                unsafe { simd::load16(NETWORK.feature_transformer.weight_ptr(src_feature, i)) };
            let dst_weights =
                unsafe { simd::load16(NETWORK.feature_transformer.weight_ptr(dst_feature, i)) };

            let new = simd::sub_i16(values, src_weights);
            let new = simd::add_i16(new, dst_weights);

            unsafe {
                simd::store16(ptr, new);
            }
        }
    }
}

impl Default for Accumulator {
    fn default() -> Self {
        Self {
            values: Align64([0; L1_SIZE]),
        }
    }
}

const COLOR_STRIDE: usize = 49;

fn gap_idx(sq: Square) -> usize {
    2 * COLOR_STRIDE + sq.idx()
}

fn piece_indices(c: Color, sq: Square) -> (usize, usize) {
    (
        c.idx() * COLOR_STRIDE + sq.idx(),
        c.flip().idx() * COLOR_STRIDE + sq.idx(),
    )
}

#[derive(Debug, Copy, Clone, Default)]
struct AccumulatorPair {
    accs: [Accumulator; 2],
}

impl AccumulatorPair {
    fn reset(&mut self, pos: &Position) {
        let biases = NETWORK.feature_transformer.biases.0.as_slice();

        self.red_mut().values.0.copy_from_slice(biases);
        self.blue_mut().values.0.copy_from_slice(biases);

        for sq in pos.gaps() {
            self.activate_gap(sq);
        }

        for sq in pos.red_occupancy() {
            self.activate_feature(Color::RED, sq);
        }

        for sq in pos.blue_occupancy() {
            self.activate_feature(Color::BLUE, sq);
        }
    }

    fn red(&self) -> &Accumulator {
        &self.accs[0]
    }

    fn blue(&self) -> &Accumulator {
        &self.accs[1]
    }

    fn red_mut(&mut self) -> &mut Accumulator {
        &mut self.accs[0]
    }

    fn blue_mut(&mut self) -> &mut Accumulator {
        &mut self.accs[1]
    }

    fn activate_gap(&mut self, sq: Square) {
        let idx = gap_idx(sq);

        self.red_mut().activate_feature(idx);
        self.blue_mut().activate_feature(idx);
    }

    pub fn activate_feature(&mut self, c: Color, sq: Square) {
        let (red_idx, blue_idx) = piece_indices(c, sq);

        self.red_mut().activate_feature(red_idx);
        self.blue_mut().activate_feature(blue_idx);
    }

    pub fn deactivate_feature(&mut self, c: Color, sq: Square) {
        let (red_idx, blue_idx) = piece_indices(c, sq);

        self.red_mut().deactivate_feature(red_idx);
        self.blue_mut().deactivate_feature(blue_idx);
    }

    pub fn move_feature(&mut self, c: Color, src_sq: Square, dst_sq: Square) {
        let (red_src_idx, blue_src_idx) = piece_indices(c, src_sq);
        let (red_dst_idx, blue_dst_idx) = piece_indices(c, dst_sq);

        self.red_mut().move_feature(red_src_idx, red_dst_idx);
        self.blue_mut().move_feature(blue_src_idx, blue_dst_idx);
    }
}

const STACK_SIZE: usize = MAX_DEPTH as usize + 1;

pub struct NnueState {
    stack: [AccumulatorPair; STACK_SIZE],
    idx: usize,
}

impl NnueState {
    pub fn reset(&mut self, pos: &Position) {
        assert_eq!(self.idx, 0);
        self.idx = 0;
        self.stack[0].reset(pos);
    }

    pub fn push(&mut self) {
        self.stack[self.idx + 1] = self.stack[self.idx];
        self.idx += 1;
    }

    pub fn pop(&mut self) -> bool {
        if self.idx == 0 {
            return false;
        }
        self.idx -= 1;
        true
    }

    fn activate_gap(&mut self, sq: Square) {
        let accs = &mut self.stack[self.idx];
        accs.activate_gap(sq);
    }

    pub fn activate_feature(&mut self, c: Color, sq: Square) {
        let accs = &mut self.stack[self.idx];
        accs.activate_feature(c, sq);
    }

    pub fn deactivate_feature(&mut self, c: Color, sq: Square) {
        let accs = &mut self.stack[self.idx];
        accs.deactivate_feature(c, sq);
    }

    pub fn move_feature(&mut self, c: Color, src_sq: Square, dst_sq: Square) {
        let accs = &mut self.stack[self.idx];
        accs.move_feature(c, src_sq, dst_sq);
    }

    pub fn evaluate(&self, stm: Color) -> Score {
        let accs = &self.stack[self.idx];
        evaluate(accs, stm)
    }
}

impl Default for NnueState {
    fn default() -> Self {
        Self {
            stack: [AccumulatorPair::default(); STACK_SIZE],
            idx: 0,
        }
    }
}

pub fn evaluate_once(pos: &Position) -> Score {
    let mut accumulator = AccumulatorPair::default();
    accumulator.reset(pos);

    evaluate(&accumulator, pos.side_to_move())
}

fn evaluate(accs: &AccumulatorPair, stm: Color) -> Score {
    let (ours, theirs) = if stm == Color::RED {
        (accs.red(), accs.blue())
    } else {
        (accs.blue(), accs.red())
    };

    let mut sum = simd::zero32();

    for i in (0..L1_SIZE).step_by(simd::CHUNK_SIZE_I16) {
        let values = unsafe { simd::load16(ours.value_ptr(i)) };
        let activated = Activation::activate(values);

        let weights = unsafe { simd::load16(NETWORK.l1.weight_ptr(0, i)) };

        let product = simd::mul_add_adj_i16(activated, weights);

        sum = simd::add_i32(product, sum);
    }

    for i in (0..L1_SIZE).step_by(simd::CHUNK_SIZE_I16) {
        let values = unsafe { simd::load16(theirs.value_ptr(i)) };
        let activated = Activation::activate(values);

        let weights = unsafe { simd::load16(NETWORK.l1.weight_ptr(L1_SIZE, i)) };

        let product = simd::mul_add_adj_i16(activated, weights);

        sum = simd::add_i32(product, sum);
    }

    simd::horizontal_sum_i32(sum) * SCALE / (L1_Q * OUTPUT_Q)
}
