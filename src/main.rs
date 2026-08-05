mod board;
mod engine;
mod operator;
mod piece;
mod undo;

use board::Board;
use engine::Search;
use std::io::{self, Write};

fn main() {
    let mut board = Board::new();
    let mut search = Search::new();
    let mut info_message = String::from(
        "Game started. Enter moves as: from_row from_col to_row to_col (e.g., 2 1 3 2). \
         Type 'ai' for the engine to move, 'eval' to see the static evaluation, or 'quit' to exit.",
    );

    loop {
        print!("\x1B[2J\x1B[1;1H");
        board.display();
        println!("-------------------------------------------------------");
        println!("Turn: {:?}", board.current_turn);
        println!("Player 1: {}", board.p1_score);
        println!("Player 2: {}", board.p2_score);
        println!("Info: {}", info_message);
        println!("-------------------------------------------------------");

        print!("Move (or 'ai' / 'eval' / 'quit'): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            info_message = String::from("Error reading input. Please try again.");
            continue;
        }

        let input = input.trim();

        if input.eq_ignore_ascii_case("quit") {
            println!("Exiting DaMath engine. Goodbye!");
            break;
        }

        if input.eq_ignore_ascii_case("eval") {
            let score = engine::evaluate(&board);
            info_message = format!(
                "Static eval (from {:?}'s perspective): {}",
                board.current_turn, score
            );
            continue;
        }

        if input.eq_ignore_ascii_case("ai") {
            match search.find_best_move(&mut board, 6, 3000) {
                Some((mv, score)) => {
                    match board.make_move(mv.from_row, mv.from_col, mv.to_row, mv.to_col) {
                        Ok(_) => {
                            info_message = format!(
                                "Engine played ({}, {}) -> ({}, {}) [eval {}]",
                                mv.from_row, mv.from_col, mv.to_row, mv.to_col, score
                            );
                        }
                        Err(err) => {
                            // Should not normally happen: it would mean
                            // engine's generated move disagreed with
                            // make_move's own validation.
                            info_message =
                                format!("Engine move was rejected by make_move: {}", err);
                        }
                    }
                }
                None => {
                    info_message = String::from("Engine found no legal moves (game over?).");
                }
            }
            continue;
        }

        let parts: Vec<i32> = input
            .split_whitespace()
            .filter_map(|s| s.parse::<i32>().ok())
            .collect();

        if parts.len() != 4 {
            info_message = String::from(
                "Invalid format! Enter 4 numbers separated by spaces (e.g. 2 1 3 2), or 'ai'/'eval'/'quit'.",
            );
            continue;
        }

        let from_row = parts[0];
        let from_col = parts[1];
        let to_row = parts[2];
        let to_col = parts[3];

        match board.make_move(from_row, from_col, to_row, to_col) {
            Ok(_) => {
                info_message = format!(
                    "Successfully moved from ({}, {}) to ({}, {}).",
                    from_row, from_col, to_row, to_col
                );
            }
            Err(err) => {
                info_message = format!("{}", err);
            }
        }
    }
}
