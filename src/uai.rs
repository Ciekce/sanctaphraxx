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
use crate::perft::{perft, split_perft};
use crate::position::{FenError::*, Position};
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

    fn handle_d(&self) {
        println!("{}", self.pos);
        println!();
        println!("Fen: {}", self.pos.to_fen());
        println!("Key: {:16x}", self.pos.key());
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
