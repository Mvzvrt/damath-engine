mod board;
mod operator;
mod piece;

use board::Board;
use std::io::{self, Write};

fn main() {
    let mut board = Board::new();
    let mut info_message = String::from(
        "Game started. Enter moves as: from_row from_col to_row to_col (e.g., 2 1 3 2)",
    );

    loop {
        // Clear terminal screen and reset cursor to top-left (ANSI escape code)
        print!("\x1B[2J\x1B[1;1H");

        // Display the current board
        board.display();

        // Print Status / Error info section
        println!("-------------------------------------------------------");
        println!("Player 1: {}", board.p1_score);
        println!("Player 2: {}", board.p2_score);
        println!("Info: {}", info_message);
        println!("-------------------------------------------------------");

        // Check for quit condition or prompt for input
        print!("Move (or type 'quit'): ");
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

        // Parse coordinates from input: expect 4 space-separated integers
        let parts: Vec<i32> = input
            .split_whitespace()
            .filter_map(|s| s.parse::<i32>().ok())
            .collect();

        if parts.len() != 4 {
            info_message =
                String::from("Invalid format! Enter 4 numbers separated by spaces (e.g. 2 1 3 2).");
            continue;
        }

        let from_row = parts[0];
        let from_col = parts[1];
        let to_row = parts[2];
        let to_col = parts[3];

        // Execute the move
        match board.make_move(from_row, from_col, to_row, to_col) {
            Ok(()) => {
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
