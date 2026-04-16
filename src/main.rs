use std::{alloc::System, io::{stdin, stdout}, process::{ExitCode, ExitStatus}, thread::sleep, time::Duration};

use crossterm::{cursor::{MoveRight, MoveTo}, execute, terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}};
use rusty_chess_bit::ChessBoard;



fn update_chess(board: &ChessBoard) {
    execute!(stdout(), MoveTo(0, 0)).unwrap();
    execute!(stdout(), Clear(ClearType::FromCursorDown)).unwrap();
    print!("{}", board);
}
fn main() -> Result<(), Box<dyn std::error::Error>>{
    execute!(stdout(), EnterAlternateScreen).unwrap();
    ctrlc::set_handler(|| {
        execute!(stdout(), LeaveAlternateScreen).unwrap();
        std::process::exit(1);
    }).expect("Error setting handler");


    let mut chess = ChessBoard::new();

    chess.set_piece(rusty_chess_bit::piece::ChessPiece::King(rusty_chess_bit::piece::Team::Black), 42);

    chess.load_initial_position();

    let mut buf = String::new();
    
    'main: loop {
        update_chess(&chess);
        
        print!(">");
        execute!(stdout(), MoveRight(1)).unwrap();

        buf.clear();
        stdin().read_line(&mut buf).expect("Error");
        let arguments = Vec::from_iter(buf.split_ascii_whitespace());
        if arguments.len() == 1 {
            if arguments.get(0).unwrap().eq(&"q") {
                break 'main;
            }
        }

        
    }


    execute!(stdout(), LeaveAlternateScreen).unwrap();
    Ok(())
}
