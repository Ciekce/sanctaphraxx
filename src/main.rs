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

use crate::bench::{run_bench, DEFAULT_BENCH_DEPTH};
use crate::search::Searcher;
use std::env;

mod ataxx_move;
mod attacks;
mod bench;
mod bitboard;
mod core;
mod eval;
mod hash;
mod limit;
mod movegen;
mod perft;
mod position;
mod search;
mod ttable;
mod uai;
mod util;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1].as_str() == "bench" {
        let mut searcher = Searcher::new();
        run_bench(&mut searcher, DEFAULT_BENCH_DEPTH);
        return;
    }

    uai::run();
}
