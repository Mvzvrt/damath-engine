use nub::board::Board;
use nub::board::GameOutcome;
use nub::engine::Search;
use nub::piece::Player;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

const ENGINE_DEPTH: u32 = 24;
const ENGINE_TIME_LIMIT: u64 = 8_000;
const ENGINE_VS_ENGINE_MIN_MOVE_DELAY: Duration = Duration::from_secs(1);
const EXIT_MESSAGE: &str = "\nBye!";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SessionMode {
    HumanVsHuman,
    HumanVsEngine,
    EngineVsEngine,
    Analysis,
    Quit,
}

fn hero_art() -> &'static str {
    r#"
██████╗  █████╗ ███╗   ███╗ █████╗ ██╗  ██╗
██╔══██╗██╔══██╗████╗ ████║██╔══██╗╚██╗██╔╝
██║  ██║███████║██╔████╔██║███████║ ╚███╔╝ 
██║  ██║██╔══██║██║╚██╔╝██║██╔══██║ ██╔██╗ 
██████╔╝██║  ██║██║ ╚═╝ ██║██║  ██║██╔╝ ██╗
╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝ by Mvzvrt
"#
}

fn hero_subtitle() -> &'static str {
    r#"
╔══════════════════════════════════════════════════════════════════════════════╗
║  A handcrafted evaluation engine for Integer Damath inspired by Pre-NNUE     ║
║  Stockfish architecture.                                                     ║
╚══════════════════════════════════════════════════════════════════════════════╝
"#
}

fn game_over_art(outcome: GameOutcome) -> &'static str {
    match outcome {
        GameOutcome::Player1Win => {
            r#"
██████╗ ██╗      █████╗ ██╗   ██╗███████╗██████╗   ███╗   ██╗    ██╗██╗███╗   ██╗██████╗ ██╗
██╔══██╗██║     ██╔══██╗╚██╗ ██╔╝██╔════╝██╔══██╗  ╚██║   ██║    ██║██║████╗  ██║██╔════╝ ██║
██████╔╝██║     ███████║ ╚████╔╝ █████╗  ██████╔╝   ██║   ██║ █╗ ██║██║██╔██╗ ██║███████╗ ██║
██╔═══╝ ██║     ██╔══██║  ╚██╔╝  ██╔══╝  ██╔══██╗   ██║   ██║███╗██║██║██║╚██╗██║╚════██║ ╚═╝
██║     ███████╗██║  ██║   ██║   ███████╗██║  ██║   ██║   ╚███╔███╔╝██║██║ ╚████║███████║ ██╗
╚═╝     ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚══════╝╚═╝  ╚═╝   ╚═╝    ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝╚══════╝ ╚═╝
"#
        }
        GameOutcome::Player2Win => {
            r#"
██████╗ ██╗      █████╗ ██╗   ██╗███████╗██████╗  ██████╗ ██╗    ██╗██╗███╗   ██╗██████╗ ██╗
██╔══██╗██║     ██╔══██╗╚██╗ ██╔╝██╔════╝██╔══██╗ ╚════██╗██║    ██║██║████╗  ██║██╔════╝ ██║
██████╔╝██║     ███████║ ╚████╔╝ █████╗  ██████╔╝  █████╔╝██║ █╗ ██║██║██╔██╗ ██║███████╗ ██║
██╔═══╝ ██║     ██╔══██║  ╚██╔╝  ██╔══╝  ██╔══██╗ ██╔═══╝ ██║███╗██║██║██║╚██╗██║╚════██║ ╚═╝
██║     ███████╗██║  ██║   ██║   ███████╗██║  ██║ ███████╗╚███╔███╔╝██║██║ ╚████║███████║ ██╗
╚═╝     ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚══════╝╚═╝  ╚═╝ ╚══════╝ ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝╚══════╝ ╚═╝
"#
        }
        GameOutcome::Draw => {
            r#"
██████╗ ██████╗  █████╗ ██╗    ██╗██╗
██╔══██╗██╔══██╗██╔══██╗██║    ██║██║
██║  ██║██████╔╝███████║██║ █╗ ██║██║
██║  ██║██╔══██╗██╔══██║██║███╗██║╚═╝
██████╔╝██║  ██║██║  ██║╚███╔███╔╝██╗
╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝ ╚══╝╚══╝ ╚═╝
"#
        }
    }
}

fn final_scores(board: &Board) -> (i32, i32) {
    (
        board.p1_score + board.remaining_piece_value(Player::Player1),
        board.p2_score + board.remaining_piece_value(Player::Player2),
    )
}

fn format_game_over(board: &Board, outcome: GameOutcome) -> String {
    let (p1_final, p2_final) = final_scores(board);

    match outcome {
        GameOutcome::Player2Win => {
            format!("{} to {}", p2_final, p1_final)
        }
        _ => {
            format!("{} to {}", p1_final, p2_final)
        }
    }
}

fn print_start_screen() {
    print!("\x1B[2J\x1B[1;1H");
    println!("{}", hero_art());
    println!("{}", hero_subtitle());
    println!("Choose a mode:");
    println!();
    println!("  1) Play new game");
    println!("  2) Play against engine");
    println!("  3) Watch engine against engine");
    println!("  4) Analysis");
    println!("  5) Quit");
    println!();
}

fn prompt_mode() -> SessionMode {
    loop {
        print!("Select mode [1-5]: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        match input.trim() {
            "1" => return SessionMode::HumanVsHuman,
            "2" => return SessionMode::HumanVsEngine,
            "3" => return SessionMode::EngineVsEngine,
            "4" => return SessionMode::Analysis,
            "5" => return SessionMode::Quit,
            _ => {
                println!("Please enter 1, 2, 3, 4, or 5.");
            }
        }
    }
}

fn print_board_frame(board: &Board, info_message: &str, prompt: Option<&str>) {
    print!("\x1B[2J\x1B[1;1H");
    board.display();
    println!("-------------------------------------------------------");
    println!("Turn: {:?}", board.current_turn);
    println!("Player 1: {}", board.p1_score);
    println!("Player 2: {}", board.p2_score);
    println!("Info: {}", info_message);
    println!("-------------------------------------------------------");
    if let Some(prompt) = prompt {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
    }
}

fn print_game_over_screen(board: &Board) {
    let outcome = board.terminal_outcome().unwrap_or(GameOutcome::Draw);

    print!("\x1B[2J\x1B[1;1H");
    println!("{}", game_over_art(outcome));
    println!("{}", format_game_over(board, outcome));
    println!();
    println!("Press (Enter) to return to the main menu.");
    io::stdout().flush().unwrap();
}

fn apply_engine_move(
    board: &mut Board,
    search: &mut Search,
    verbose: bool,
    depth: u32,
    time_limit_ms: u64,
) -> Option<String> {
    let (mv, score) = search.find_best_move(board, depth, time_limit_ms, verbose)?;

    match board.make_move(mv.from_row, mv.from_col, mv.to_row, mv.to_col) {
        Ok(_) => Some(format!(
            "Computer ({}, {}) -> ({}, {}) [Score {}].",
            mv.from_row, mv.from_col, mv.to_row, mv.to_col, score
        )),
        Err(err) => Some(format!("Engine move was rejected by make_move: {}", err)),
    }
}

fn apply_engine_move_with_min_delay(
    board: &mut Board,
    search: &mut Search,
    depth: u32,
    time_limit_ms: u64,
    min_move_delay: Duration,
) -> Option<String> {
    let start = Instant::now();
    let (mv, score) = search.find_best_move(board, depth, time_limit_ms, false)?;

    let elapsed = start.elapsed();
    if elapsed < min_move_delay {
        thread::sleep(min_move_delay - elapsed);
    }

    match board.make_move(mv.from_row, mv.from_col, mv.to_row, mv.to_col) {
        Ok(_) => Some(format!(
            "{:?} played ({}, {}) -> ({}, {}) [Score {}]. {:?} thinking...",
            board.current_turn.opponent(),
            mv.from_row,
            mv.from_col,
            mv.to_row,
            mv.to_col,
            score,
            board.current_turn.opponent().opponent()
        )),
        Err(err) => Some(format!("Engine move was rejected by make_move: {}", err)),
    }
}

fn run_human_vs_human(board: &mut Board) {
    let mut info_message = String::from(
        "[Game] Enter moves as: from_row from_col to_row to_col. Type 'quit' to return to main menu.",
    );

    loop {
        if board.terminal_outcome().is_some() {
            print_game_over_screen(board);
            let mut pause = String::new();
            io::stdin().read_line(&mut pause).ok();
            break;
        }

        print_board_frame(board, &info_message, Some("Move: "));

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            info_message = String::from("Error reading input. Please try again.");
            continue;
        }

        let input = input.trim();
        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!("{}", EXIT_MESSAGE);
            break;
        }

        let parts: Vec<i32> = input
            .split_whitespace()
            .filter_map(|s| s.parse::<i32>().ok())
            .collect();

        if parts.len() != 4 {
            info_message = String::from("Invalid format. Enter 4 numbers separated by spaces.");
            continue;
        }

        match board.make_move(parts[0], parts[1], parts[2], parts[3]) {
            Ok(_) => {
                info_message = String::from("Move applied.");
            }
            Err(err) => {
                info_message = err.to_string();
            }
        }

        if board.terminal_outcome().is_some() {
            print_game_over_screen(board);
            let mut pause = String::new();
            io::stdin().read_line(&mut pause).ok();
            break;
        }
    }
}

fn run_human_vs_engine(board: &mut Board, search: &mut Search) {
    let mut info_message = String::from(
        "Enter moves as: from_row from_col to_row to_col. Type 'quit' to return to main menu.",
    );

    loop {
        if board.terminal_outcome().is_some() {
            print_game_over_screen(board);
            let mut pause = String::new();
            io::stdin().read_line(&mut pause).ok();
            break;
        }

        print_board_frame(
            board,
            &info_message,
            if board.current_turn == Player::Player1 {
                Some("Your move: ")
            } else {
                None
            },
        );

        if board.current_turn == Player::Player2 {
            info_message = apply_engine_move(board, search, false, ENGINE_DEPTH, ENGINE_TIME_LIMIT)
                .unwrap_or_else(|| String::from("Engine found no legal moves (game over?)."));
            if board.terminal_outcome().is_some() {
                print_game_over_screen(board);
                let mut pause = String::new();
                io::stdin().read_line(&mut pause).ok();
                break;
            }
            continue;
        }

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            info_message = String::from("Error reading input. Please try again.");
            continue;
        }

        let input = input.trim();
        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!("{}", EXIT_MESSAGE);
            break;
        }

        let parts: Vec<i32> = input
            .split_whitespace()
            .filter_map(|s| s.parse::<i32>().ok())
            .collect();

        if parts.len() != 4 {
            info_message = String::from("Invalid format. Enter 4 numbers separated by spaces.");
            continue;
        }

        match board.make_move(parts[0], parts[1], parts[2], parts[3]) {
            Ok(_) => {
                info_message = String::from("Move applied. Engine thinking...");
            }
            Err(err) => {
                info_message = err.to_string();
                continue;
            }
        }

        if board.terminal_outcome().is_some() {
            print_game_over_screen(board);
            let mut pause = String::new();
            io::stdin().read_line(&mut pause).ok();
            break;
        }
    }
}

fn run_engine_vs_engine(board: &mut Board, search: &mut Search) {
    let mut info_message =
        String::from("[Computer vs. Computer] Both sides are played by the computer.");

    loop {
        if board.terminal_outcome().is_some() {
            print_game_over_screen(board);
            let mut pause = String::new();
            io::stdin().read_line(&mut pause).ok();
            break;
        }

        print_board_frame(board, &info_message, None);

        info_message = apply_engine_move_with_min_delay(
            board,
            search,
            ENGINE_DEPTH,
            ENGINE_TIME_LIMIT,
            ENGINE_VS_ENGINE_MIN_MOVE_DELAY,
        )
        .unwrap_or_else(|| String::from("Computer found no legal moves (game over?)."));

        if board.terminal_outcome().is_some() {
            print_game_over_screen(board);
            let mut pause = String::new();
            io::stdin().read_line(&mut pause).ok();
            break;
        }
    }
}

fn run_analysis(board: &mut Board, search: &mut Search) {
    let mut info_message =
        String::from("[Analysis] Press Enter for the best move, or enter a move to play it.");

    loop {
        if board.terminal_outcome().is_some() {
            print_game_over_screen(board);
            let mut pause = String::new();
            io::stdin().read_line(&mut pause).ok();
            break;
        }

        print_board_frame(
            board,
            &info_message,
            Some("Enter for best move, type a move, or 'quit': "),
        );

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        let input = input.trim();
        if input.eq_ignore_ascii_case("quit") {
            break;
        }

        if input.is_empty() {
            print!("\x1B[2J\x1B[1;1H");
            board.display();
            println!(
                "Analyzing (depth up to {}, {}s budget)...\n",
                ENGINE_DEPTH, ENGINE_TIME_LIMIT
            );

            match search.find_best_move(board, ENGINE_DEPTH, ENGINE_TIME_LIMIT, true) {
                Some((mv, score)) => {
                    println!(
                        "[Best move]: ({}, {}) -> ({}, {}) [score {}]",
                        mv.from_row, mv.from_col, mv.to_row, mv.to_col, score
                    );
                }
                None => {
                    println!("Analysis found no legal moves (game over?).");
                }
            }

            println!("\nPress Enter to continue...");
            let mut pause = String::new();
            io::stdin().read_line(&mut pause).ok();
            info_message = String::from(
                "Analysis mode. Press Enter for the best move, or enter a move to play it.",
            );
            continue;
        }

        let parts: Vec<i32> = input
            .split_whitespace()
            .filter_map(|s| s.parse::<i32>().ok())
            .collect();

        if parts.len() != 4 {
            info_message = String::from(
                "Invalid format. Press Enter for a suggestion, or enter 4 numbers to play a move.",
            );
            continue;
        }

        match board.make_move(parts[0], parts[1], parts[2], parts[3]) {
            Ok(_) => {
                info_message = String::from("Move applied. Press Enter to analyze again.");
            }
            Err(err) => {
                info_message = err.to_string();
                continue;
            }
        }

        if board.terminal_outcome().is_some() {
            print_game_over_screen(board);
            let mut pause = String::new();
            io::stdin().read_line(&mut pause).ok();
            break;
        }
    }
}

fn main() {
    let mut search = Search::new();
    loop {
        print_start_screen();
        let mode = prompt_mode();
        if mode == SessionMode::Quit {
            println!("{}", EXIT_MESSAGE);
            break;
        }

        let mut board = Board::new();

        match mode {
            SessionMode::HumanVsHuman => run_human_vs_human(&mut board),
            SessionMode::HumanVsEngine => run_human_vs_engine(&mut board, &mut search),
            SessionMode::EngineVsEngine => run_engine_vs_engine(&mut board, &mut search),
            SessionMode::Analysis => run_analysis(&mut board, &mut search),
            SessionMode::Quit => unreachable!(),
        }
    }
}
