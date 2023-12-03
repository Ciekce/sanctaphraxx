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

use crate::ataxx_move::{AtaxxMove, MoveStrError};
use crate::core::{Color, MAX_DEPTH};
use crate::eval::static_eval;
use crate::limit::SearchLimiter;
use crate::perft::{perft, split_perft};
use crate::position::{FenError::*, Position};
use crate::search::search_root;
use std::str::FromStr;

const NAME: &str = "Sanctaphraxx";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

struct UaiHandler {
    pos: Position,
}

impl UaiHandler {
    #[must_use]
    fn new() -> Self {
        Self {
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
                "isready" => self.handle_isready(),
                "position" => self.handle_position(&cmd[1..]),
                "go" => self.handle_go(&cmd[1..]),
                "d" => self.handle_d(),
                "perft" => self.handle_perft(&cmd[1..]),
                "splitperft" => self.handle_splitperft(&cmd[1..]),
                "quit" => break,
                unknown => eprintln!("Unknown command '{}'", unknown),
            }

            line.clear();
        }
    }

    fn handle_uai(&self) {
        println!("id name {} {}", NAME, VERSION);
        println!("id author {}", AUTHORS.replace(':', ", "));
        //TODO options
        println!("uaiok");
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
                    match err {
                        NotEnoughParts => eprintln!("Incomplete FEN"),
                        NotEnoughRanks => eprintln!("Not enough ranks in FEN"),
                        TooManyRanks => eprintln!("Too many ranks in FEN"),
                        NotEnoughFiles(rank) => eprintln!("Not enough files in rank {}", rank + 1),
                        TooManyFiles(rank) => eprintln!("Too many files in rank {}", rank + 1),
                        InvalidChar(c) => eprintln!("Invalid character '{}' in FEN", c),
                        InvalidStm => eprintln!("Invalid side to move in FEN"),
                        InvalidHalfmove => eprintln!("Invalid halfmove clock in FEN"),
                        InvalidFullmove => eprintln!("Invalid fullmove number in FEN"),
                    }
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
                Ok(m) => self.pos.apply_move::<false, true>(m),
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

    fn handle_go(&self, args: &[&str]) {
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

                    let value = if let Ok(value) = args[i].parse::<u64>() {
                        value
                    } else {
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

        search_root(self.pos.clone(), &mut limiter.unwrap(), depth);
    }

    fn handle_d(&self) {
        println!("{}", self.pos);
        println!();
        println!("Fen: {}", self.pos.to_fen());
        println!("Key: {:16x}", self.pos.key());
        println!("Static eval: {}", static_eval(&self.pos));
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
}

pub fn run() {
    let mut handler = UaiHandler::new();
    handler.run();
}
