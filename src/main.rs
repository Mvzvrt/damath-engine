mod board;
mod engine;
mod operator;
mod piece;
mod undo;
mod zobrist;

use board::Board;
use engine::Search;
use piece::Player;
use std::io::{self, Write};

fn format_game_over(board: &Board) -> String {
    let p1_final = board.p1_score + board.remaining_piece_value(Player::Player1);
    let p2_final = board.p2_score + board.remaining_piece_value(Player::Player2);

    match p1_final.cmp(&p2_final) {
        std::cmp::Ordering::Greater => format!(
            "Game over. Player 1 wins. Final scores: {} to {}.",
            p1_final, p2_final
        ),
        std::cmp::Ordering::Less => format!(
            "Game over. Player 2 wins. Final scores: {} to {}.",
            p1_final, p2_final
        ),
        std::cmp::Ordering::Equal => format!(
            "Game over. Draw. Final scores: {} to {}.",
            p1_final, p2_final
        ),
    }
}

fn main() {
    let mut board = Board::new();
    let mut search = Search::new();
    let mut info_message = String::from(
        "Game started. Enter moves as: from_row from_col to_row to_col. \
         'ai' lets the engine move, 'analyze' shows live search scores \
         without moving, 'newgame' resets everything, or 'quit'.",
    );

    loop {
        if board.terminal_outcome().is_some() {
            info_message = format_game_over(&board);
            print!("\x1B[2J\x1B[1;1H");
            board.display();
            println!("-------------------------------------------------------");
            println!("Turn: {:?}", board.current_turn);
            println!("Player 1: {}", board.p1_score);
            println!("Player 2: {}", board.p2_score);
            println!("Info: {}", info_message);
            println!("-------------------------------------------------------");
            break;
        }

        print!("\x1B[2J\x1B[1;1H");
        board.display();
        println!("-------------------------------------------------------");
        println!("Turn: {:?}", board.current_turn);
        println!("Player 1: {}", board.p1_score);
        println!("Player 2: {}", board.p2_score);
        println!("Info: {}", info_message);
        println!("-------------------------------------------------------");

        print!("Move (or 'ai' / 'analyze' / 'newgame' / 'quit'): ");
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

        if input.eq_ignore_ascii_case("newgame") {
            board = Board::new();
            search.reset();
            info_message = String::from("New game started, search state cleared.");
            continue;
        }

        if input.eq_ignore_ascii_case("analyze") {
            print!("\x1B[2J\x1B[1;1H");
            board.display();
            println!("Analyzing (depth up to 24, 8s budget)...\n");

            match search.find_best_move(&mut board, 24, 8000) {
                Some((mv, score)) => {
                    info_message = format!(
                        "Analysis complete. Best line starts ({}, {}) -> ({}, {}), score {}. \
                         (Not played — enter a move manually or type 'ai'.)",
                        mv.from_row, mv.from_col, mv.to_row, mv.to_col, score
                    );
                }
                None => {
                    info_message = String::from("Analysis found no legal moves (game over?).");
                }
            }

            println!("\nPress Enter to continue...");
            let mut _pause = String::new();
            io::stdin().read_line(&mut _pause).ok();
            continue;
        }

        if input.eq_ignore_ascii_case("ai") {
            match search.find_best_move(&mut board, 20, 4000) {
                Some((mv, score)) => {
                    match board.make_move(mv.from_row, mv.from_col, mv.to_row, mv.to_col) {
                        Ok(_) => {
                            info_message = format!(
                                "Engine played ({}, {}) -> ({}, {}) [searched score {}]",
                                mv.from_row, mv.from_col, mv.to_row, mv.to_col, score
                            );
                        }
                        Err(err) => {
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
                "Invalid format! Enter 4 numbers separated by spaces (e.g. 2 1 3 2), or 'ai'/'analyze'/'newgame'/'quit'.",
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

        if board.terminal_outcome().is_some() {
            info_message = format_game_over(&board);
        }
    }
}