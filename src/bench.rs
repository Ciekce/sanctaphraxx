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

use crate::position::Position;
use crate::search::Searcher;

pub const DEFAULT_BENCH_DEPTH: i32 = 6;
pub const BENCH_TT_SIZE: usize = 16;

const BENCH_FENS: &[&str] = &[
    "x-1-1-o/-1-1-1-/1-1-1-1/-1-1-1-/1-1-1-1/-1-1-1-/o-1-1-x x 0 1",
    "x-1-1-o/1-1-1-1/1-1-1-1/1-1-1-1/1-1-1-1/1-1-1-1/o-1-1-x x 0 1",
    "x1-1-1o/2-1-2/-------/2-1-2/-------/2-1-2/o1-1-1x x 0 1",
    "x5o/1-----1/1-3-1/1-1-1-1/1-3-1/1-----1/o5x x 0 1",
    "x-1-1-o/1-1-1-1/-1-1-1-/-1-1-1-/-1-1-1-/1-1-1-1/o-1-1-x x 0 1",
    "x5o/1--1--1/1--1--1/7/1--1--1/1--1--1/o5x x 0 1",
    "x-3-o/1-1-1-1/1-1-1-1/3-3/1-1-1-1/1-1-1-1/o-3-x x 0 1",
    "x2-2o/3-3/3-3/-------/3-3/3-3/o2-2x x 0 1",
    "x2-2o/2-1-2/1-3-1/-2-2-/1-3-1/2-1-2/o2-2x x 0 1",
    "x5o/7/7/7/7/7/o5x x 0 1",
    "x5o/7/2-1-2/7/2-1-2/7/o5x x 0 1",
    "x5o/7/3-3/2-1-2/3-3/7/o5x x 0 1",
    "x2-2o/3-3/2---2/7/2---2/3-3/o2-2x x 0 1",
    "x2-2o/3-3/7/--3--/7/3-3/o2-2x x 0 1",
    "x1-1-1o/2-1-2/2-1-2/7/2-1-2/2-1-2/o1-1-1x x 0 1",
    "x5o/7/2-1-2/3-3/2-1-2/7/o5x x 0 1",
    "x5o/7/3-3/2---2/3-3/7/o5x x 0 1",
    "x5o/2-1-2/1-3-1/7/1-3-1/2-1-2/o5x x 0 1",
    "x5o/1-3-1/2-1-2/7/2-1-2/1-3-1/o5x x 0 1",
    "2x3o/7/7/7/o6/5x1/6x o 2 2",
    "5oo/7/x6/x6/7/7/o5x o 0 2",
    "x5o/1x5/7/7/7/2o4/4x2 o 0 2",
    "7/7/2x1o2/1x5/7/7/o5x o 0 2",
    "7/7/1x4o/7/4x2/7/o6 o 3 2",
    "x5o/7/6x/7/1o5/7/7 o 3 2",
    "5oo/7/2x4/7/7/4x2/o6 o 1 2",
    "x5o/7/7/3x3/7/1o5/o6 o 1 2",
    "x5o/7/7/7/7/2x1x2/3x3 o 0 2",
    "7/7/1x4o/7/7/4x2/o6 o 3 2",
    "x5o/7/7/5x1/5x1/1o5/o6 o 0 2",
    "6o/7/4x2/7/7/1o5/o5x o 1 2",
    "x5o/x5o/7/7/7/6x/o5x o 0 2",
    "4x1o/7/7/7/7/o6/o5x o 1 2",
    "6o/7/x6/7/7/2o4/6x o 3 2",
    "x5o/7/7/7/1o4x/7/5x1 o 2 2",
    "x5o/6o/7/7/4x2/7/o6 o 1 2",
    "7/7/1xx1o2/7/7/7/o5x o 0 2",
    "2x3o/2x4/7/7/7/7/2o3x o 0 2",
    "x5o/6o/7/7/4x2/3x3/o6 o 0 2",
    "x5o/7/7/7/o3xx1/7/7 o 0 2",
    "6o/6o/1x5/7/4x2/7/o6 o 1 2",
    "7/7/4x1o/7/7/7/o5x o 3 2",
    "4o2/7/2x4/7/7/7/o4xx o 0 2",
    "2x3o/x6/7/7/7/o6/o5x o 1 2",
    "6o/7/2x4/7/1o5/7/4x2 o 3 2",
    "x6/4o2/7/7/6x/7/o6 o 3 2",
    "x6/7/5o1/7/7/4x2/o6 o 3 2",
    "x5o/1x4o/7/7/7/7/o3x2 o 0 2",
    "xx4o/7/7/7/7/6x/oo4x o 0 2",
    "x6/7/4x2/3x3/7/7/o5x o 2 2",
];

pub fn run_bench(searcher: &mut Searcher, depth: i32) {
    searcher.resize_tt(BENCH_TT_SIZE);
    println!("set TT size to {} MB", BENCH_TT_SIZE);

    let mut total_nodes = 0usize;
    let mut total_time = 0f64;

    let mut pos = Position::empty();

    for fen in BENCH_FENS {
        if let Err(err) = pos.reset_from_fen(fen) {
            eprintln!("Invalid bench fen {}", fen);
            eprintln!("{}", err);
        }

        searcher.new_game();

        let (nodes, time) = searcher.bench(&mut pos, depth);

        total_nodes += nodes;
        total_time += time;
    }

    let nps = (total_nodes as f64 / total_time) as usize;

    println!("{:.2} seconds", total_time);
    println!("{} nodes {} nps", total_nodes, nps);
}
