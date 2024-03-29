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
use crate::core::*;
use crate::eval::static_eval;
use crate::limit::SearchLimiter;
use crate::movegen::{fill_scored_move_list, ScoredMoveList};
use crate::nnue::NnueState;
use crate::position::{GameResult, Position};
use crate::ttable::{TTable, TtEntryFlag};
use std::time::Instant;

pub struct SearchContext<'a> {
    pub pos: &'a mut Position,
    pub nnue_state: NnueState,
    pub nodes: usize,
    pub seldepth: u32,
    pub best_move: AtaxxMove,
}

impl<'a> SearchContext<'a> {
    pub fn new(pos: &'a mut Position) -> Self {
        Self {
            pos,
            nnue_state: NnueState::default(),
            nodes: 0,
            seldepth: 0,
            best_move: AtaxxMove::None,
        }
    }
}

pub struct Searcher {
    limiter: SearchLimiter,
    ttable: TTable,
}

impl Searcher {
    #[must_use]
    pub fn new() -> Self {
        Self {
            limiter: SearchLimiter::infinite(),
            ttable: TTable::new(),
        }
    }

    pub fn new_game(&mut self) {
        self.ttable.clear();
    }

    pub fn resize_tt(&mut self, mb: usize) {
        self.ttable.resize(mb);
    }

    pub fn start_search(&mut self, mut pos: Position, limiter: SearchLimiter, max_depth: i32) {
        self.limiter = limiter;

        let mut ctx = SearchContext::new(&mut pos);
        ctx.nnue_state.reset(ctx.pos);

        self.search_root(&mut ctx, max_depth, true);
    }

    pub fn run_datagen_search(
        &mut self,
        ctx: &mut SearchContext,
        limiter: SearchLimiter,
        max_depth: i32,
    ) -> Score {
        self.limiter = limiter;

        let score = self.search_root(ctx, max_depth, false);

        if ctx.pos.side_to_move() == Color::BLUE {
            -score
        } else {
            score
        }
    }

    #[must_use]
    pub fn bench(&mut self, pos: &mut Position, depth: i32) -> (usize, f64) {
        self.limiter = SearchLimiter::infinite();

        let mut ctx = SearchContext::new(pos);
        ctx.nnue_state.reset(ctx.pos);

        let start = Instant::now();

        self.search_root(&mut ctx, depth, false);

        let time = start.elapsed().as_secs_f64();
        (ctx.nodes, time)
    }

    fn search_root(&mut self, ctx: &mut SearchContext, max_depth: i32, report: bool) -> Score {
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

        score
    }

    #[must_use]
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
            return static_eval(ctx.pos, &ctx.nnue_state);
        }

        let is_root = ply == 0;
        let is_pv = beta - alpha > 1;

        let tt_entry = self.ttable.probe(ctx.pos.key()).unwrap_or_default();
        let tt_hit = tt_entry.flag != TtEntryFlag::None;

        if !is_pv
            && tt_hit
            && i32::from(tt_entry.depth) >= depth
            && match tt_entry.flag {
                TtEntryFlag::Exact => true,
                TtEntryFlag::Alpha => Score::from(tt_entry.score) <= alpha,
                TtEntryFlag::Beta => Score::from(tt_entry.score) >= beta,
                TtEntryFlag::None => unreachable!(),
            }
        {
            return Score::from(tt_entry.score);
        }

        // if no tt hit, the entry's move is None
        let tt_move = tt_entry.mv.unpack();

        let mut moves = ScoredMoveList::new();
        fill_scored_move_list(&mut moves, ctx.pos);
        Self::order_moves(&mut moves, tt_move);

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
        let mut best_move = AtaxxMove::None;

        let mut entry_flag = TtEntryFlag::Alpha;

        for (move_idx, &(mv, _)) in moves.iter().enumerate() {
            ctx.nodes += 1;

            ctx.pos.apply_move::<true, true>(
                mv,
                if mv != AtaxxMove::Null {
                    Some(&mut ctx.nnue_state)
                } else {
                    None
                },
            );

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

            ctx.pos.pop_move::<true>(if mv != AtaxxMove::Null {
                Some(&mut ctx.nnue_state)
            } else {
                None
            });

            if score > best_score {
                best_score = score;

                if score > alpha {
                    best_move = mv;

                    if is_root {
                        ctx.best_move = mv;
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
                .store(ctx.pos.key(), best_move, best_score, depth, entry_flag);
        }

        best_score
    }

    // very temporary solution
    //TODO movepicker
    fn order_moves(moves: &mut ScoredMoveList, tt_move: AtaxxMove) {
        for (mv, score) in moves.iter_mut() {
            if *mv == tt_move {
                *score = 100;
                break;
            }
        }

        moves.sort_unstable_by(|(_, a_score), (_, b_score)| b_score.cmp(a_score));
    }

    fn report(ctx: &SearchContext, mv: AtaxxMove, depth: i32, time: f64, score: Score) {
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
            mv
        );
    }
}
