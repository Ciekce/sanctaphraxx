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

use crate::ataxx_move::{AtaxxMove, MoveStrError};
use crate::bench::{run_bench, DEFAULT_BENCH_DEPTH};
use crate::core::{Color, MAX_DEPTH};
use crate::eval::static_eval_once;
use crate::limit::SearchLimiter;
use crate::perft::{perft, split_perft};
use crate::position::Position;
use crate::search::Searcher;
use crate::ttable::TTable;
use std::str::FromStr;

const NAME: &str = "Sanctaphraxx";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

struct UaiHandler {
    searcher: Searcher,
    pos: Position,
}

#[allow(clippy::unused_self)]
impl UaiHandler {
    #[must_use]
    fn new() -> Self {
        Self {
            searcher: Searcher::new(),
            pos: Position::startpos(),
        }
    }

    fn run(&mut self) {
        let mut line = String::with_capacity(256);
        while let Ok(bytes) = std::io::stdin().read_line(&mut line) {
            if bytes == 0 {
                break;
            }

            let cmd: Vec<&str> = line.split_whitespace().collect();
            if cmd.is_empty() {
                line.clear();
                continue;
            }

            match cmd[0] {
                "uai" => self.handle_uai(),
                "uainewgame" => self.handle_uainewgame(),
                "setoption" => self.handle_setoption(&cmd[1..]),
                "isready" => self.handle_isready(),
                "position" => self.handle_position(&cmd[1..]),
                "go" => self.handle_go(&cmd[1..]),
                "d" => self.handle_d(),
                "perft" => self.handle_perft(&cmd[1..]),
                "splitperft" => self.handle_splitperft(&cmd[1..]),
                "bench" => self.handle_bench(&cmd[1..]),
                "quit" => break,
                unknown => eprintln!("Unknown command '{}'", unknown),
            }

            line.clear();
        }
    }

    fn handle_uai(&self) {
        println!("id name {} {}", NAME, VERSION);
        println!("id author {}", AUTHORS.replace(':', ", "));
        println!(
            "option name Hash type spin default {} min {} max {}",
            TTable::DEFAULT_SIZE_MB,
            TTable::MIN_SIZE_MB,
            TTable::MAX_SIZE_MB
        );
        println!("uaiok");
    }

    fn handle_uainewgame(&mut self) {
        self.searcher.new_game();
    }

    fn handle_setoption(&mut self, args: &[&str]) {
        if args.len() < 2 || args[0] != "name" {
            eprintln!("Missing name");
            return;
        }

        let mut idx = 1usize;
        while idx < args.len() && args[idx] != "value" {
            idx += 1;
        }

        if idx > args.len() - 2 || args[idx] != "value" {
            eprintln!("Missing value");
            return;
        }

        let name = args[1usize..idx].join(" ");
        let value = args[(idx + 1)..].join(" ");

        #[allow(clippy::single_match)]
        match name.as_str() {
            "Hash" => {
                if let Ok(new_size) = value.parse::<usize>() {
                    self.searcher.resize_tt(new_size);
                } else {
                    eprintln!("Invalid hash size");
                }
            }
            _ => {}
        }
    }

    fn handle_isready(&self) {
        println!("readyok");
    }

    fn handle_position(&mut self, args: &[&str]) {
        if args.is_empty() {
            return;
        }

        let next = match args[0] {
            "startpos" => {
                self.pos.reset_to_startpos();
                1usize
            }
            "fen" => {
                if let Err(err) = self.pos.reset_from_fen_parts(&args[1..]) {
                    eprintln!("{}", err);
                    return;
                }
                5usize
            }
            _ => return,
        };

        if args.len() <= next {
            return;
        } else if args[next] != "moves" {
            eprintln!("Unknown token '{}'", args[next]);
            return;
        }

        for move_str in &args[next + 1..] {
            match AtaxxMove::from_str(move_str) {
                Ok(m) => self.pos.apply_move::<false, true>(m, None),
                Err(err) => eprintln!(
                    "Invalid move '{}': {}",
                    move_str,
                    match err {
                        MoveStrError::InvalidFrom => "invalid from-square",
                        MoveStrError::InvalidTo => "invalid to-square",
                        MoveStrError::WrongSize => "wrong size",
                    }
                ),
            }
        }
    }

    fn handle_go(&mut self, args: &[&str]) {
        let mut limiter: Option<SearchLimiter> = None;
        let mut depth = MAX_DEPTH;

        let mut tournament_time = false;

        let mut red_time = 0u64;
        let mut blue_time = 0u64;
        let mut red_inc = 0u64;
        let mut blue_inc = 0u64;

        let mut moves_to_go = 0u64;

        let mut i = 0usize;
        while i < args.len() {
            match args[i] {
                "infinite" => {
                    if tournament_time || limiter.is_some() {
                        eprintln!("Multiple non-depth search limits not supported");
                        return;
                    }

                    limiter = Some(SearchLimiter::infinite());
                }
                "depth" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("Missing depth");
                        return;
                    }

                    depth = if let Ok(depth) = args[i].parse::<i32>() {
                        depth
                    } else {
                        eprintln!("Invalid depth '{}'", args[i]);
                        return;
                    }
                }
                "nodes" => {
                    if tournament_time || limiter.is_some() {
                        eprintln!("Multiple non-depth search limits not supported");
                        return;
                    }

                    i += 1;
                    if i >= args.len() {
                        eprintln!("Missing node count");
                        return;
                    }

                    if let Ok(node_limit) = args[i].parse::<usize>() {
                        limiter = Some(SearchLimiter::fixed_nodes(node_limit));
                    } else {
                        eprintln!("Invalid node limit '{}'", args[i]);
                        return;
                    }
                }
                "movetime" => {
                    if tournament_time || limiter.is_some() {
                        eprintln!("Multiple non-depth search limits not supported");
                        return;
                    }

                    i += 1;
                    if i >= args.len() {
                        eprintln!("Missing move time");
                        return;
                    }

                    if let Ok(time_limit) = args[i].parse::<u64>() {
                        limiter = Some(SearchLimiter::move_time(time_limit));
                    } else {
                        eprintln!("Invalid move time '{}'", args[i]);
                        return;
                    }
                }
                "wtime" | "btime" | "winc" | "binc" | "movestogo" => {
                    if limiter.is_some() {
                        eprintln!("Multiple non-depth search limits not supported");
                        return;
                    }

                    tournament_time = true;

                    let token = args[i];

                    i += 1;
                    if i >= args.len() {
                        eprintln!("Missing {}", token);
                        return;
                    }

                    let Ok(value) = args[i].parse::<u64>() else {
                        eprintln!("Invalid {} '{}'", token, args[i]);
                        return;
                    };

                    match token {
                        "wtime" => blue_time = value,
                        "btime" => red_time = value,
                        "winc" => blue_inc = value,
                        "binc" => red_inc = value,
                        "movestogo" => moves_to_go = value,
                        _ => unreachable!(),
                    }
                }
                unknown => {
                    eprintln!("Unknown search limit '{}'", unknown);
                    return;
                }
            }

            i += 1;
        }

        if tournament_time {
            assert!(limiter.is_none());

            let (our_time, our_inc) = match self.pos.side_to_move() {
                Color::RED => (red_time, red_inc),
                Color::BLUE => (blue_time, blue_inc),
                _ => unreachable!(),
            };

            limiter = Some(SearchLimiter::tournament(our_time, our_inc, moves_to_go));
        } else if limiter.is_none() {
            limiter = Some(SearchLimiter::infinite());
        }

        self.searcher
            .start_search(self.pos.clone(), limiter.unwrap(), depth);
    }

    fn handle_d(&self) {
        println!("{}", self.pos);
        println!();
        println!("Fen: {}", self.pos.to_fen());
        println!("Key: {:16x}", self.pos.key());
        println!("Static eval: {}", static_eval_once(&self.pos));
    }

    fn handle_perft(&mut self, args: &[&str]) {
        if args.is_empty() {
            eprintln!("Missing depth");
            return;
        }

        if let Ok(depth) = args[0].parse::<i32>() {
            perft(&mut self.pos, depth);
        } else {
            eprintln!("Invalid depth");
        }
    }

    fn handle_splitperft(&mut self, args: &[&str]) {
        if args.is_empty() {
            eprintln!("Missing depth");
            return;
        }

        if let Ok(depth) = args[0].parse::<i32>() {
            split_perft(&mut self.pos, depth);
        } else {
            eprintln!("Invalid depth");
        }
    }

    fn handle_bench(&mut self, args: &[&str]) {
        let depth = if args.is_empty() {
            DEFAULT_BENCH_DEPTH
        } else if let Ok(depth) = args[0].parse::<i32>() {
            depth
        } else {
            eprintln!("Invalid depth");
            return;
        };

        run_bench(&mut self.searcher, depth);
    }
}

pub fn run() {
    let mut handler = UaiHandler::new();
    handler.run();
}
