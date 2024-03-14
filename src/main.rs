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

use crate::bench::{run_bench, DEFAULT_BENCH_DEPTH};
use crate::search::Searcher;
use std::env;
use std::process::exit;

mod ataxx_move;
mod attacks;
mod bench;
mod bitboard;
mod core;
mod datagen;
mod eval;
mod hash;
mod limit;
mod movegen;
mod nnue;
mod perft;
mod position;
mod search;
mod ttable;
mod uai;
mod util;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "bench" => {
                let mut searcher = Searcher::new();
                run_bench(&mut searcher, DEFAULT_BENCH_DEPTH);
                return;
            }
            "datagen" => {
                if args.len() < 4 {
                    eprintln!(
                        "usage: {} datagen <fens|bulletformat> <path> [threads] [game limit per thread]",
                        args[0]
                    );
                    exit(1);
                }

                let write_fens = match args[2].as_str() {
                    "fens" => true,
                    "bulletformat" => false,
                    _ => {
                        eprintln!("invalid output format {}", args[3]);
                        eprintln!(
                            "usage: {} datagen <fens|bulletformat> <path> [threads] [game limit per thread]",
                            args[0]
                        );
                        exit(1);
                    }
                };

                let threads = args
                    .get(4)
                    .map(|arg| {
                        if let Ok(threads) = arg.parse::<u32>() {
                            threads
                        } else {
                            eprintln!("invalid number of threads {}", arg);
                            eprintln!(
                                "usage: {} datagen <fens|bulletformat> <path> [threads] [game limit per thread]",
                                args[0]
                            );
                            exit(1);
                        }
                    })
                    .unwrap_or(1);

                let games = args
                    .get(5)
                    .map(|arg| {
                        if let Ok(games) = arg.parse::<u32>() {
                            games
                        } else {
                            eprintln!("invalid number of games {}", arg);
                            eprintln!(
                                "usage: {} datagen <fens|bulletformat> <path> [threads] [game limit per thread]",
                                args[0]
                            );
                            exit(1);
                        }
                    })
                    .unwrap_or(datagen::UNLIMITED_GAMES);

                datagen::run(args[3].as_str(), write_fens, threads, games);
                return;
            }
            _ => {}
        }
    }

    uai::run();
}
