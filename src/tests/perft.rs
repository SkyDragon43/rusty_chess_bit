


use regex::Regex;

use crate::{ChessBoard, tests::suite};

//mod suite;


fn perft(board: &mut ChessBoard, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = board.generate_moves();
    let mut count = 0;
    for chess_move in moves {
        board.play_move(chess_move);
        count += perft(board, depth - 1);
        board.undo_move();
    }
    count
} 
fn perft_moves(board: &mut ChessBoard, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = board.generate_moves();
    let mut count = 0;
    println!("{:5} {} at depth {}","Move","Nodes",depth);
    for chess_move in moves {
        let played_move_name = board.play_move(chess_move).original.name();

        let move_count = perft(board, depth - 1);

        count += move_count;
        println!("{}: {}",played_move_name,move_count);
        board.undo_move();
    }
    println!("Total visited nodes was {}", count);
    count
} 
#[test]
pub fn test_perft() {
    
    let mut chess = ChessBoard::new();

    chess.load_initial_position();

    assert_eq!(perft(&mut chess, 1), 20);

    assert_eq!(perft(&mut chess, 2), 400);

    assert_eq!(perft(&mut chess, 3), 8902);

    assert_eq!(perft(&mut chess, 4), 197281);

    assert_eq!(perft(&mut chess, 5), 4865609);

    
}

#[test]
/// https://www.chessprogramming.org/Perft_Results#Position_3
fn test_position_3() {
    let mut chess = ChessBoard::new();

    chess.load_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();

    assert_eq!(perft_moves(&mut chess, 1), 14);

    assert_eq!(perft_moves(&mut chess, 2), 191);

    assert_eq!(perft_moves(&mut chess, 3), 2812);

    assert_eq!(perft_moves(&mut chess, 4), 43238);

    assert_eq!(perft_moves(&mut chess, 5), 674624);
}


#[test]
fn test_perft_castles() {
    let mut chess = ChessBoard::new();

    chess.load_fen("r4k1r/Pppp1ppp/1b3nbN/nPP5/BB2P3/q4N2/Pp1P2PP/1R1Q1RK1 b - - 0 1").unwrap();
    
    assert_eq!(perft_moves(&mut chess, 1), 34);


    chess.load_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();

    assert_eq!(perft_moves(&mut chess, 1), 6);

    assert_eq!(perft_moves(&mut chess, 2), 264);

    assert_eq!(perft_moves(&mut chess, 3), 9467);

    assert_eq!(perft_moves(&mut chess, 4), 422333);

    assert_eq!(perft_moves(&mut chess, 5), 15833292);
}
#[test]
fn test_position_5() {
    let mut chess = ChessBoard::new();

    chess.load_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();

    assert_eq!(perft_moves(&mut chess, 1), 44);

    assert_eq!(perft_moves(&mut chess, 2), 1486);

    assert_eq!(perft_moves(&mut chess, 3), 62379);

    assert_eq!(perft_moves(&mut chess, 4), 2103487);
}

#[test]
/// https://www.chessprogramming.org/Perft_Results#Position_2
fn test_position_2() {
    let mut chess = ChessBoard::new();

    chess.load_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();

    assert_eq!(perft_moves(&mut chess, 1), 48);

    assert_eq!(perft_moves(&mut chess, 2), 2039);

    assert_eq!(perft_moves(&mut chess, 3), 97862);

    assert_eq!(perft_moves(&mut chess, 4), 4085603);
}

#[test]
/// https://www.chessprogramming.org/Perft_Results#Position_4
fn test_position_4() {
    let mut chess = ChessBoard::new();

    chess.load_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();

    assert_eq!(perft_moves(&mut chess, 1), 6);

    assert_eq!(perft_moves(&mut chess, 2), 264);

    assert_eq!(perft_moves(&mut chess, 3), 9467);

    assert_eq!(perft_moves(&mut chess, 4), 422333);
}
#[test]
/// https://www.chessprogramming.org/Perft_Results#Position_6
fn test_position_6() {
    let mut chess = ChessBoard::new();

    chess.load_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10").unwrap();

    assert_eq!(perft_moves(&mut chess, 1), 46);

    assert_eq!(perft_moves(&mut chess, 2), 2079);

    assert_eq!(perft_moves(&mut chess, 3), 89890);

    assert_eq!(perft_moves(&mut chess, 4), 3894594);
}
#[test]
fn test_position_7() {
    let mut chess = ChessBoard::new();

    chess.load_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/2KR3R b qk - 0 1").unwrap();

    assert_eq!(perft_moves(&mut chess, 1), 43);

    assert_eq!(perft_moves(&mut chess, 2), 1887);

    assert_eq!(perft_moves(&mut chess, 3), 79803);

    assert_eq!(perft_moves(&mut chess, 4), 3551583);
}

#[test]
fn test_perft_2() {

    let mut chess = ChessBoard::new();

    chess.load_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/P1N2Q2/1PPBBPpP/1R2K2R b Kqk - 0 2").unwrap();

    assert_eq!(perft_moves(&mut chess, 2), 2201);
}
#[test]
fn test_suite() {

    let mut chess = ChessBoard::new();

    //let file = read("perftsuite.txt").unwrap();
    let tests = suite::TESTING_SUITE;
    let lines = tests.lines();


    let regex = Regex::new(r"^(?<fen>(?:[pnbrqkPNBRQK1-8]{1,8}\/){7}[pnbrqkPNBRQK1-8]{1,8}\s+(?:b|w)\s+(?:-|[K|Q|k|q]{1,4})\s+(?:-|[a-h][36])\s+\d+\s+\d+)\s+(?<depths>(?:;D\d\s+\d+\s*)*)$").unwrap();

    for line in lines {
        let capture = regex.captures(&line);
        if capture.is_none() {
            continue;
        }
        let capture = capture.unwrap();

        let fen = capture.name("fen").unwrap().as_str();
        chess.load_fen(fen).unwrap();

        let depths = capture.name("depths").unwrap().as_str();
        let mut depths = depths.split_ascii_whitespace();
        let count = depths.clone().count();
        assert!(count % 2 == 0);

        println!("Testing from testing suite: '{}'", fen);
        for _ in 0..count / 2 {
            let depth = depths.next().unwrap();
            let depth = &depth[2..]; // trim leading ;D
            let depth: u8 = depth.parse().unwrap();

            let node_count: u64 = depths.next().unwrap().parse().unwrap();

            assert_eq!(perft(&mut chess, depth), node_count);
        }
    }

}