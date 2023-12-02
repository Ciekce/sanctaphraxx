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
use crate::core::{Score, MAX_DEPTH, SCORE_INF, SCORE_MATE, SCORE_WIN};
use crate::eval::static_eval;
use crate::movegen::{generate_moves, MoveList};
use crate::position::{GameResult, Position};
use std::time::Instant;

struct SearchContext<'a> {
    pos: &'a mut Position,
    nodes: usize,
    seldepth: u32,
    best_move: AtaxxMove,
}

pub fn search_root(mut pos: Position, depth: i32) {
    let mut ctx = SearchContext {
        pos: &mut pos,
        nodes: 0,
        seldepth: 0,
        best_move: AtaxxMove::None,
    };

    let start = Instant::now();

    let score = search(&mut ctx, depth, 0);

    let time = start.elapsed().as_secs_f64();
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
        ctx.best_move
    );
    println!("bestmove {}", ctx.best_move);
}

fn search(ctx: &mut SearchContext, depth: i32, ply: i32) -> Score {
    ctx.seldepth = ctx.seldepth.max(ply as u32);

    if depth <= 0 || ply >= MAX_DEPTH {
        return static_eval(ctx.pos);
    }

    let root = ply == 0;

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

    for m in moves {
        ctx.nodes += 1;

        ctx.pos.apply_move::<true, true>(m);
        let score = search(ctx, depth - 1, ply + 1);
        ctx.pos.pop_move::<true>();

        if score > best_score {
            if root {
                ctx.best_move = m;
            }

            best_score = score;
        }
    }

    best_score
}
