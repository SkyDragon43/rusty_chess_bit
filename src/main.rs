use std::{alloc::System, env, io::{Read, stdin, stdout}, process::{ExitCode, ExitStatus}, thread::sleep, time::Duration};

use crossterm::{cursor::{MoveRight, MoveTo}, event::{self, Event}, execute, terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}};
use rand::random_range;
use rusty_chess_bit::ChessBoard;



fn update_chess(board: &ChessBoard) {
    execute!(stdout(), MoveTo(0, 0)).unwrap();
    execute!(stdout(), Clear(ClearType::FromCursorDown)).unwrap();
    print!("{}", board);
}
fn main() -> Result<(), Box<dyn std::error::Error>>{
    unsafe { env::set_var("RUST_BACKTRACE", "full") };

    execute!(stdout(), EnterAlternateScreen).unwrap();
    ctrlc::set_handler(|| {
        execute!(stdout(), LeaveAlternateScreen).unwrap();
        std::process::exit(1);
    }).expect("Error setting handler");


    let mut chess = ChessBoard::new();

    chess.set_piece(rusty_chess_bit::piece::ChessPiece::King(rusty_chess_bit::piece::Team::Black), 42);

    chess.load_initial_position();
    //chess.load_fen("rnb1kbnr/p1q3pp/1p1p4/2p1NpB1/Q1PP2P1/5P2/PP2P2P/RN2KB1R b KQkq - 0 1");
    chess.load_fen("rnb1kbnr/p1q3pp/1p1pN3/2p2pB1/Q1PP2P1/5P2/PP2P2P/RN2KB1R b KQkq - 0 1");
    chess.load_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();
    
    let mut buf = String::new();
    
    let mut message = String::new();
    'main: loop {
        let mut moves = chess.generate_moves();
        let move_count = moves.len();
        //update_chess(&chess);
        execute!(stdout(), MoveTo(0, 0)).unwrap();
        execute!(stdout(), Clear(ClearType::FromCursorDown)).unwrap();
        

        message.push_str(&format!("There are {} moves\n", move_count));
        for (i, line) in message.lines().enumerate() {
            execute!(stdout(), MoveTo(37, 2 + i as u16)).unwrap();
            print!("{}", line);
        }
        message.clear();

        execute!(stdout(), MoveTo(0, 0)).unwrap();
        print!("{}", chess);
        print!(">");

        execute!(stdout(), MoveRight(1)).unwrap();

        buf.clear();
        stdin().read_line(&mut buf).expect("Error");
        let arguments = Vec::from_iter(buf.split_ascii_whitespace());
        if arguments.len() == 1 {
            if arguments.get(0).unwrap().eq(&"q") {
                break 'main;
            } else if arguments.get(0).unwrap().eq(&"r") {
                if move_count > 0 {
                    message.push_str("Played random move. ");
                    let to_play = moves.swap_remove(random_range(0..move_count));
                    chess.play_move(to_play);
                } else {
                    message.push_str("No more moves to play. ");
                }
            } else if arguments.get(0).unwrap().eq(&"u") {
                if chess.undo_move().is_some() {
                    message.push_str("Undoed move. ");
                } else {
                    message.push_str("No more moves to undo. ");
                }
            } else if arguments.get(0).unwrap().eq(&"l") {
                for l in moves {
                    message += &l.name();
                    message += "\n";
                }
            } else if arguments.get(0).unwrap().eq(&"fen") {
                message += "Fen string is :\n";
                message += &chess.to_fen();
                message += "\n";
            } else {
                let m = arguments.get(0).unwrap();
                moves.retain(|f| {
                    f.name().starts_with(m)
                });
                if moves.len() == 0 {
                    message.push_str(&format!("Unkown move '{}'", m));
                } else if moves.len() == 1 {
                    let m = moves.remove(0);
                    chess.play_move(m);
                } else {
                    message.push_str(&format!("Valid moves:"));
                    for m in moves {
                        message.push_str(&format!(" '{}'", m.name()));
                    }
                    message.push_str(" ");
                }
            }
        } else if arguments.len() > 1 {
            if arguments.get(0).unwrap().eq(&"fen") {
                let mut iter = arguments.iter();
                iter.next();
                let mut fen = String::new();
                for s in iter {
                    fen.push_str(s);
                    fen.push(' ');
                }
                match chess.load_fen(&fen) {
                    Ok(_) => message.push_str("Successfully loaded fen string.\n"),
                    Err(e) => message.push_str(&format!("Unable to load fen: {}\n", fen)),
                };
            }
        }
    }

    while chess.undo_move().is_some() {
        update_chess(&chess);
        sleep(Duration::from_millis(20));
    }
    println!("Press enter to exit.");

    let mut buf = [0u8; 1];
    let _ = stdin().read(&mut buf);


    execute!(stdout(), LeaveAlternateScreen).unwrap();
    Ok(())
}
