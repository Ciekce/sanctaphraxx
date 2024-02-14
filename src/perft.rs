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

use crate::movegen::{fill_move_list, MoveList};
use crate::position::Position;
use std::time::Instant;

#[must_use]
fn do_perft(pos: &mut Position, depth: i32) -> usize {
    if depth <= 0 {
        return 1;
    }

    let mut moves = MoveList::new();
    fill_move_list(&mut moves, pos);

    if depth == 1 {
        return moves.len();
    }

    let mut total = 0usize;

    for m in moves {
        pos.apply_move::<true, false>(m, None);
        total += do_perft(pos, depth - 1);
        pos.pop_move::<false>(None);
    }

    total
}

pub fn perft(pos: &mut Position, depth: i32) {
    let total = do_perft(pos, depth);
    println!("{}", total);
}

pub fn split_perft(pos: &mut Position, depth: i32) {
    let start = Instant::now();

    let mut moves = MoveList::new();
    fill_move_list(&mut moves, pos);

    let mut total = 0usize;

    for m in moves {
        pos.apply_move::<true, false>(m, None);

        let value = do_perft(pos, depth - 1);

        total += value;
        println!("{}\t{}", m, value);

        pos.pop_move::<false>(None);
    }

    let time = start.elapsed().as_secs_f64();
    let nps = (total as f64 / time) as usize;

    println!();
    println!("total {}", total);
    println!("{} nps", nps);
}

#[cfg(test)]
mod tests {
    use crate::perft::do_perft;
    use crate::position::Position;

    #[rustfmt::skip]
    const PERFT4_POSITIONS: &[(&str, &[usize])] = &[
        ("7/7/7/7/7/7/7 x 0 1", &[1, 0, 0, 0, 0]),
        ("7/7/7/7/7/7/7 o 0 1", &[1, 0, 0, 0, 0]),
        ("7/7/7/7/ooooooo/ooooooo/xxxxxxx o 0 1", &[1, 75, 249, 14270, 452980]),
        ("7/7/7/7/xxxxxxx/xxxxxxx/ooooooo x 0 1", &[1, 75, 249, 14270, 452980]),
        ("x5o/7/7/7/7/7/o5x x 100 1", &[1, 0, 0, 0, 0]),
        ("x5o/7/7/7/7/7/o5x o 100 1", &[1, 0, 0, 0, 0]),
    ];

    #[rustfmt::skip]
    const PERFT5_POSITIONS: &[(&str,  &[usize])] = &[
        ("x5o/7/7/7/7/7/o5x x 0 1", &[1, 16, 256, 6460, 155888, 4752668]),
        ("x5o/7/7/7/7/7/o5x o 0 1", &[1, 16, 256, 6460, 155888, 4752668]),
        ("x5o/7/2-1-2/7/2-1-2/7/o5x x 0 1", &[1, 14, 196, 4184, 86528, 2266352]),
        ("x5o/7/2-1-2/7/2-1-2/7/o5x o 0 1", &[1, 14, 196, 4184, 86528, 2266352]),
        ("x5o/7/2-1-2/3-3/2-1-2/7/o5x x 0 1", &[1, 14, 196, 4100, 83104, 2114588]),
        ("x5o/7/2-1-2/3-3/2-1-2/7/o5x o 0 1", &[1, 14, 196, 4100, 83104, 2114588]),
        ("x5o/7/3-3/2-1-2/3-3/7/o5x x 0 1", &[1, 16, 256, 5948, 133264, 3639856]),
        ("x5o/7/3-3/2-1-2/3-3/7/o5x o 0 1", &[1, 16, 256, 5948, 133264, 3639856]),
        ("7/7/7/7/ooooooo/ooooooo/xxxxxxx x 0 1", &[1, 1, 75, 249, 14270, 452980]),
        ("7/7/7/7/xxxxxxx/xxxxxxx/ooooooo o 0 1", &[1, 1, 75, 249, 14270, 452980]),
        ("7/7/7/2x1o2/7/7/7 x 0 1", &[1, 23, 419, 7887, 168317, 4266992]),
        ("7/7/7/2x1o2/7/7/7 o 0 1", &[1, 23, 419, 7887, 168317, 4266992]),
    ];

    #[rustfmt::skip]
    const PERFT6_POSITIONS: &[(&str, &[usize])] = &[
        ("7/7/7/7/-------/-------/x5o x 0 1", &[1, 2, 4, 13, 30, 73, 174]),
        ("7/7/7/7/-------/-------/x5o o 0 1", &[1, 2, 4, 13, 30, 73, 174]),
    ];

    fn test_perft(positions: &[(&str, &[usize])]) {
        let mut pos = Position::empty();

        for (fen, counts) in positions {
            pos.reset_from_fen(fen).unwrap();
            for (depth, &count) in counts.iter().enumerate() {
                assert_eq!(do_perft(&mut pos, depth as i32), count);
            }
        }
    }

    #[test]
    fn perft4() {
        test_perft(PERFT4_POSITIONS);
    }

    #[test]
    fn perft5() {
        test_perft(PERFT5_POSITIONS);
    }

    #[test]
    fn perft6() {
        test_perft(PERFT6_POSITIONS);
    }
}
