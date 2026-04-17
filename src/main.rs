use std::{alloc::System, env, io::{Read, stdin, stdout}, process::{ExitCode, ExitStatus}, thread::sleep, time::Duration};

use crossterm::{cursor::{MoveRight, MoveTo}, event::{self, Event, MouseEvent}, execute, style::Print, terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}};
use rand::random_range;
use rusty_chess_bit::ChessBoard;



fn update_chess(board: &ChessBoard, messages: &str) {
    execute!(stdout(), MoveTo(0, 0)).unwrap();
    execute!(stdout(), Clear(ClearType::FromCursorDown)).unwrap();
    
    let mut y: i8 = 2;
    for (i, line) in messages.lines().enumerate() {
        let wrapped = textwrap::wrap(line, 45);
        let height = wrapped.len();
        for (j, wrapped) in textwrap::wrap(line, 33).iter().enumerate() {
            if y >= 0 {
                execute!(stdout(), MoveTo(37, y as u16)).unwrap();
                print!("{}", wrapped);
            }
            y += 1;
        }
        
    }

    execute!(stdout(), MoveTo(0, 0)).unwrap();
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

    
    let mut buf = String::new();
    
    let mut message = String::new();
    'main: loop {
        let mut moves = chess.generate_moves();
        let move_count = moves.len();

        update_chess(&chess, &message);
        message.clear();

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
                message.push_str(&format!("There are {} moves.\n", move_count));
                for l in moves {
                    message += &format!("'{}'", &l.name());
                    message += " ";
                }
            } else if arguments.get(0).unwrap().eq(&"fen") {
                message += "Fen string is :\n";
                message += &chess.to_fen();
                message += "\n";
            } else if arguments[0].eq("reset") {
                while chess.undo_move().is_some() {
                    update_chess(&chess, "Resetting...");
                    sleep(Duration::from_millis(10));
                }
                message.push_str("Board reset.");
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
                    Err(e) => message.push_str(&format!("Unable to load: {}", e)),
                };
            } else if arguments[0].eq("r") {
                match u32::from_str_radix(arguments[1], 10) {
                    Ok(e) => {
                        for i in 0..e {
                            let mut moves = chess.generate_moves();
                            if moves.len() > 0 {
                                let to_play = moves.swap_remove(random_range(0..moves.len()));
                                chess.play_move(to_play);
                                update_chess(&chess, "Playing moves...");
                                sleep(Duration::from_millis(10));
                            } else {
                                message.push_str("No more moves.\n");
                                break;
                            }
                        }
                        message.push_str("Finished playing moves!");
                    },
                    Err(e) => message.push_str(&format!("{}", e)),
                };
            }
        }
    }

    execute!(stdout(), LeaveAlternateScreen).unwrap();
    Ok(())
}
