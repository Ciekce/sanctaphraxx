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

use crate::ataxx_move::AtaxxMove;
use crate::bitboard::Bitboard;
use crate::core::{Color, Score, MAX_DEPTH, SCORE_WIN};
use crate::limit::SearchLimiter;
use crate::movegen::{fill_move_list, MoveList};
use crate::position::{GameResult, Position};
use crate::search::{SearchContext, Searcher};
use crate::util::rng::Jsf64Rng;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const UNLIMITED_GAMES: u32 = u32::MAX;

const TT_SIZE: usize = 64;

const NODE_LIMIT: usize = 5000;

const VERIFICATION_DEPTH: i32 = 4;
const VERIFICATION_SCORE_LIMIT: Score = SCORE_WIN;

const WIN_ADJ_MIN_SCORE: Score = 2500;
const DRAW_ADJ_MAX_SCORE: Score = 10;

const WIN_ADJ_MAX_PLIES: u32 = 5;
const DRAW_ADJ_MAX_PLIES: u32 = 5;

const REPORT_INTERVAL: u32 = 1024;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum Outcome {
    RedLoss = 0,
    Draw,
    RedWin,
}

impl Outcome {
    fn flip(self) -> Self {
        match self {
            Self::RedLoss => Self::RedWin,
            Self::Draw => Self::Draw,
            Self::RedWin => Self::RedLoss,
        }
    }
}

trait OutputFormat {
    type Elem;

    const EXTENSION: &'static str;

    fn pack(pos: &Position, red_score: Score) -> Self::Elem;
    fn write_all_with_outcome(out: &mut impl Write, values: &mut [Self::Elem], outcome: Outcome);
}

struct Fen;
impl OutputFormat for Fen {
    type Elem = String;

    const EXTENSION: &'static str = "txt";

    fn pack(pos: &Position, red_score: Score) -> String {
        format!("{} | {}", pos.to_fen(), red_score)
    }

    fn write_all_with_outcome(out: &mut impl Write, values: &mut [Self::Elem], outcome: Outcome) {
        for fen in values {
            writeln!(
                out,
                "{} | {}",
                fen,
                match outcome {
                    Outcome::RedLoss => "0.0",
                    Outcome::Draw => "0.5",
                    Outcome::RedWin => "1.0",
                }
            )
            .unwrap();
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct BulletFormat {
    bbs: [u64; 3],
    score: i16,
    result: Outcome,
    stm: bool,
    fullmoves: u16,
    halfmoves: u8,
    extra: u8,
}

impl OutputFormat for BulletFormat {
    type Elem = Self;

    const EXTENSION: &'static str = "bin";

    fn pack(pos: &Position, red_score: Score) -> Self {
        #[allow(clippy::unreadable_literal)]
        fn to_bullet_bb(board: Bitboard) -> u64 {
            #[cfg(target_feature = "bmi2")]
            {
                use core::arch::x86_64::*;
                unsafe { _pext_u64(board.raw(), Bitboard::ALL.raw()) }
            }

            #[cfg(not(target_feature = "bmi2"))]
            {
                let bb = board.raw();
                bb & 0x7f
                    | (bb & 0x7f00) >> 1
                    | (bb & 0x7f0000) >> 2
                    | (bb & 0x7f000000) >> 3
                    | (bb & 0x7f00000000) >> 4
                    | (bb & 0x7f0000000000) >> 5
                    | (bb & 0x7f000000000000) >> 6
            }
        }

        let (stm_occ, nstm_occ, stm_score) = if pos.side_to_move() == Color::RED {
            (pos.red_occupancy(), pos.blue_occupancy(), red_score)
        } else {
            (pos.blue_occupancy(), pos.red_occupancy(), -red_score)
        };

        let stm_occ = to_bullet_bb(stm_occ);
        let nstm_occ = to_bullet_bb(nstm_occ);

        Self {
            bbs: [stm_occ, nstm_occ, pos.gaps().raw()],
            score: stm_score as i16,
            result: Outcome::RedLoss,
            stm: pos.side_to_move() == Color::BLUE,
            fullmoves: pos.fullmoves() as u16,
            halfmoves: pos.halfmoves() as u8,
            extra: 0,
        }
    }

    fn write_all_with_outcome(out: &mut impl Write, values: &mut [Self::Elem], outcome: Outcome) {
        for board in values.iter_mut() {
            board.result = if board.stm {
                // blue
                outcome.flip()
            } else {
                outcome
            };
        }

        let written = out
            .write(unsafe {
                std::slice::from_raw_parts(
                    values.as_ptr().cast::<u8>(),
                    std::mem::size_of_val(values),
                )
            })
            .unwrap();
        assert_eq!(written, std::mem::size_of_val(values));
    }
}

static STOP: AtomicBool = AtomicBool::new(false);

fn run_thread<T: OutputFormat>(id: u32, games: u32, seed: u64, out_dir: &Path) {
    let out_path = out_dir.join(format!("{}.{}", id, T::EXTENSION));
    let Ok(out_file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(out_path.as_path())
    else {
        eprintln!("Failed to open output file {}", out_path.to_str().unwrap());
        return;
    };

    let mut out = BufWriter::new(out_file);

    let mut rng = Jsf64Rng::new(seed);

    let verif_limiter = SearchLimiter::infinite();
    let limiter = SearchLimiter::fixed_nodes(NODE_LIMIT);

    let mut searcher = Searcher::new();
    searcher.resize_tt(TT_SIZE);

    let mut pos = Position::empty();
    let mut ctx = SearchContext::new(&mut pos);

    let mut positions = Vec::<T::Elem>::new();

    let start_time = Instant::now();

    let mut total_positions = 0usize;

    let mut game = 0;
    while game < games {
        positions.clear();
        searcher.new_game();

        ctx.pos.reset_to_startpos();

        let move_count = 8 + (rng.next_u32() >> 31);
        let mut hit_game_over = false;

        for _ in 0..move_count {
            let mut moves = MoveList::new();
            fill_move_list(&mut moves, ctx.pos);

            if moves.is_empty() {
                hit_game_over = true;
                break;
            }

            let mv = moves[rng.next_u32_bounded(moves.len() as u32) as usize];
            ctx.pos.apply_move::<false, true>(mv, None);

            if ctx.pos.game_over() {
                hit_game_over = true;
                break;
            }
        }

        if hit_game_over {
            continue;
        }

        let first_score =
            searcher.run_datagen_search(&mut ctx, verif_limiter.clone(), VERIFICATION_DEPTH);
        if first_score.abs() > VERIFICATION_SCORE_LIMIT {
            continue;
        }

        searcher.new_game();

        let outcome: Outcome;

        let mut win_plies = 0u32;
        let mut loss_plies = 0u32;
        let mut draw_plies = 0u32;

        loop {
            ctx.nnue_state.reset(ctx.pos);
            let score = searcher.run_datagen_search(&mut ctx, limiter.clone(), MAX_DEPTH);
            assert_ne!(ctx.best_move, AtaxxMove::None);

            if score.abs() > SCORE_WIN {
                outcome = if score > 0 {
                    Outcome::RedWin
                } else {
                    Outcome::RedLoss
                };
                break;
            }

            if score > WIN_ADJ_MIN_SCORE {
                win_plies += 1;
                loss_plies = 0;
                draw_plies = 0;
            } else if score < -WIN_ADJ_MIN_SCORE {
                win_plies = 0;
                loss_plies += 1;
                draw_plies = 0;
            } else if score.abs() < DRAW_ADJ_MAX_SCORE {
                win_plies = 0;
                loss_plies = 0;
                draw_plies += 1;
            } else {
                win_plies = 0;
                loss_plies = 0;
                draw_plies = 0;
            }

            if win_plies >= WIN_ADJ_MAX_PLIES {
                outcome = Outcome::RedWin;
                break;
            } else if loss_plies >= WIN_ADJ_MAX_PLIES {
                outcome = Outcome::RedLoss;
                break;
            } else if draw_plies >= DRAW_ADJ_MAX_PLIES {
                outcome = Outcome::Draw;
                break;
            }

            ctx.pos.apply_move::<false, true>(ctx.best_move, None);

            if ctx.pos.game_over() {
                outcome = match ctx.pos.result() {
                    GameResult::Win(color) => {
                        if color != ctx.pos.side_to_move() {
                            Outcome::RedWin
                        } else {
                            Outcome::RedLoss
                        }
                    }
                    GameResult::Draw => Outcome::Draw,
                };
                break;
            }

            positions.push(T::pack(ctx.pos, score));
        }

        T::write_all_with_outcome(&mut out, &mut positions, outcome);

        total_positions += positions.len();

        let stop = STOP.load(Ordering::SeqCst);

        if stop || game == games - 1 || ((game + 1) % REPORT_INTERVAL) == 0 {
            let time = start_time.elapsed().as_secs_f64();
            println!(
                "thread {}: wrote {} positions from {} games in {} sec ({:.2} positions/sec)",
                id,
                total_positions,
                game + 1,
                time,
                total_positions as f64 / time
            );
        }

        if stop {
            break;
        }

        game += 1;
    }

    out.flush().unwrap();
}

#[allow(clippy::unreadable_literal)]
fn mix(mut v: u64) -> u64 {
    v ^= v >> 33;
    v = v.wrapping_mul(0xff51afd7ed558ccd);
    v ^= v >> 33;
    v = v.wrapping_mul(0xc4ceb9fe1a85ec53);
    v ^ v >> 33
}

pub fn run(output: &str, write_fens: bool, threads: u32, games: u32) {
    // extremely scuffed
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64;
    let addr = std::ptr::addr_of!(time) as u64;

    let base_seed = mix(time ^ addr);
    println!("base seed: {}", base_seed);

    let output_dir = Path::new(output);

    if let Err(err) = ctrlc::set_handler(|| {
        STOP.store(true, Ordering::SeqCst);
    }) {
        eprintln!("failed to set Ctrl+C handler: {}", err);
    }

    if games == UNLIMITED_GAMES {
        println!("generating on {} threads", threads);
    } else {
        println!("generating {} games each on {} threads", games, threads);
    }

    std::thread::scope(|s| {
        for id in 0..threads {
            s.spawn(move || {
                if write_fens {
                    run_thread::<Fen>(id, games, base_seed + u64::from(id), output_dir);
                } else {
                    run_thread::<BulletFormat>(id, games, base_seed + u64::from(id), output_dir);
                }
            });
        }
    });

    println!("done");
}
