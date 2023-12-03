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

use crate::ataxx_move::AtaxxMove;
use crate::core::*;
use crate::eval::static_eval;
use crate::limit::SearchLimiter;
use crate::movegen::{generate_moves, MoveList};
use crate::position::{GameResult, Position};
use crate::ttable::{TTable, TtEntry, TtEntryFlag};
use std::time::Instant;

pub struct SearchContext<'a> {
    pos: &'a mut Position,
    nodes: usize,
    seldepth: u32,
    best_move: AtaxxMove,
}

pub struct Searcher {
    limiter: SearchLimiter,
    ttable: TTable,
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            limiter: SearchLimiter::infinite(),
            ttable: TTable::new(),
        }
    }

    pub fn new_game(&mut self) {
        self.ttable.clear();
    }

    pub fn start_search(&mut self, mut pos: Position, limiter: SearchLimiter, max_depth: i32) {
        self.limiter = limiter;

        let mut ctx = SearchContext {
            pos: &mut pos,
            nodes: 0,
            seldepth: 0,
            best_move: AtaxxMove::None,
        };

        self.search_root(&mut ctx, max_depth, true);
    }

    pub fn bench(&mut self, pos: &mut Position, depth: i32) -> (usize, f64) {
        self.limiter = SearchLimiter::infinite();

        let mut ctx = SearchContext {
            pos,
            nodes: 0,
            seldepth: 0,
            best_move: AtaxxMove::None,
        };

        let start = Instant::now();

        self.search_root(&mut ctx, depth, false);

        let time = start.elapsed().as_secs_f64();
        (ctx.nodes, time)
    }

    fn search_root(&mut self, ctx: &mut SearchContext, max_depth: i32, report: bool) {
        assert!(max_depth > 0);

        let max_depth = max_depth.min(MAX_DEPTH);

        let start = Instant::now();

        let mut score = -SCORE_INF;
        let mut best_move = AtaxxMove::None;

        let mut depth_completed = 0i32;

        for depth in 1..=max_depth {
            ctx.seldepth = 0;

            score = self.search(ctx, -SCORE_INF, SCORE_INF, depth, 0);

            if self.limiter.stopped() {
                break;
            }

            depth_completed = depth;
            best_move = ctx.best_move;

            if report && depth < max_depth {
                let time = start.elapsed().as_secs_f64();
                Self::report(ctx, best_move, depth, time, score);
            }

            if self.limiter.should_stop(ctx.nodes) {
                break;
            }
        }

        if report {
            let time = start.elapsed().as_secs_f64();
            Self::report(ctx, best_move, depth_completed, time, score);

            println!("bestmove {}", best_move);
        }
    }

    fn search(
        &mut self,
        ctx: &mut SearchContext,
        mut alpha: Score,
        beta: Score,
        depth: i32,
        ply: i32,
    ) -> Score {
        if depth > 1 && self.limiter.should_stop(ctx.nodes) {
            return beta;
        }

        ctx.seldepth = ctx.seldepth.max(ply as u32);

        if depth <= 0 || ply >= MAX_DEPTH {
            return static_eval(ctx.pos);
        }

        let is_root = ply == 0;
        let is_pv = beta - alpha > 1;

        if !is_pv {
            if let Some(tt_entry) = self.ttable.probe(ctx.pos.key()) {
                if tt_entry.depth as i32 >= depth
                    && match tt_entry.flag {
                        TtEntryFlag::Exact => true,
                        TtEntryFlag::Alpha => tt_entry.score as i32 <= alpha,
                        TtEntryFlag::Beta => tt_entry.score as i32 >= beta,
                        _ => unreachable!(),
                    }
                {
                    return tt_entry.score as i32;
                }
            }
        }

        let mut moves = MoveList::new();
        generate_moves(&mut moves, ctx.pos);

        if moves.is_empty() {
            return match ctx.pos.result() {
                GameResult::Win(side) => {
                    if side == ctx.pos.side_to_move() {
                        SCORE_MATE - ply
                    } else {
                        -SCORE_MATE + ply
                    }
                }
                GameResult::Draw => 0,
            };
        }

        let mut best_score: Score = -SCORE_INF;
        let mut entry_flag = TtEntryFlag::Alpha;

        for (move_idx, &m) in moves.iter().enumerate() {
            ctx.nodes += 1;

            ctx.pos.apply_move::<true, true>(m);

            let score = if is_pv && move_idx == 0 {
                -self.search(ctx, -beta, -alpha, depth - 1, ply + 1)
            } else {
                let zw_score = -self.search(ctx, -alpha - 1, -alpha, depth - 1, ply + 1);
                if zw_score > alpha && zw_score < beta {
                    -self.search(ctx, -beta, -alpha, depth - 1, ply + 1)
                } else {
                    zw_score
                }
            };

            ctx.pos.pop_move::<true>();

            if score > best_score {
                best_score = score;

                if score > alpha {
                    if is_root {
                        ctx.best_move = m;
                    }

                    if score >= beta {
                        entry_flag = TtEntryFlag::Beta;
                        break;
                    }

                    alpha = score;
                    entry_flag = TtEntryFlag::Exact;
                }
            }
        }

        if !self.limiter.stopped() {
            self.ttable
                .store(ctx.pos.key(), best_score, depth, entry_flag);
        }

        best_score
    }

    fn report(ctx: &SearchContext, m: AtaxxMove, depth: i32, time: f64, score: Score) {
        let nps = (ctx.nodes as f64 / time) as usize;

        println!(
            "info depth {} seldepth {} time {} nodes {} nps {} score {} pv {}",
            depth,
            ctx.seldepth,
            (time * 1000.0) as usize,
            ctx.nodes,
            nps,
            if score.abs() > SCORE_WIN {
                format!(
                    "mate {}",
                    if score > 0 {
                        (SCORE_MATE - score + 1) / 2
                    } else {
                        -(SCORE_MATE + score) / 2
                    }
                )
            } else {
                format!("cp {}", score)
            },
            m
        );
    }
}
