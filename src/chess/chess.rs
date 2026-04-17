use std::{arch::x86_64::_MM_FROUND_TO_ZERO, borrow::Borrow, collections::VecDeque, error::Error, fmt::Display, hint::black_box, path::Iter, pin, string::ParseError};

use crossterm::{queue, style::Stylize, terminal::LeaveAlternateScreen};
use regex::{Regex};

use crate::{castle::{self, CastleRights, CastleType}, chess::{constants::{ANTI_DIAGONAL_0, ANTI_DIAGONALS, DIAGONAL_0, DIAGONALS, FILE_A, FILE_B, FILE_F, FILE_G, FILE_H, RANK_1, RANK_2, RANK_3, RANK_4, RANK_6, RANK_7, RANK_8, file_char, get_anti_diagonal, get_diagonal, index_from_string, rank_char}, piece::{ChessPiece, Team}}, moves::{ChessMove, MoveType, PlayedMove}};



fn index_coords(x: i8, y: i8) -> i8 {
    if x < 0 || y < 0 || x > 7 || y > 7 {
        return -1;
    }
    y * 8 + x
}
fn coord_mask(x: i8, y: i8) -> u64 {
    if x < 0 || y < 0 || x > 7 || y > 7 {
        return 0;
    }
    1u64 << x << (y * 8)
}



struct Pieces {
    pawns: u64,
    knights: u64,
    bishops: u64,
    rooks: u64,
    queens: u64,
    kings: u64,
    all: u64,
    pinned: [u64; 64]
}

impl Pieces {
    fn new() -> Pieces {
        Pieces { pawns: 0, knights: 0, bishops: 0, rooks: 0, queens: 0, kings: 0, all: 0, pinned: [0; 64] }
    }

    pub fn set_piece(&mut self, piece: ChessPiece, index: u8) {
        let mask = 1 << index;
        let negative_mask = !mask;
        if piece.is_none() {
            self.all &= negative_mask;
            self.pawns &= negative_mask;
            self.knights &= negative_mask;
            self.bishops &= negative_mask;
            self.rooks &= negative_mask;
            self.queens &= negative_mask;
            self.kings &= negative_mask;
        } else if piece.is_pawn() {
            self.all |= mask;
            self.pawns |= mask;
            self.knights &= negative_mask;
            self.bishops &= negative_mask;
            self.rooks &= negative_mask;
            self.queens &= negative_mask;
            self.kings &= negative_mask;
        } else if piece.is_knight() {
            self.all |= mask;
            self.pawns &= negative_mask;
            self.knights |= mask;
            self.bishops &= negative_mask;
            self.rooks &= negative_mask;
            self.queens &= negative_mask;
            self.kings &= negative_mask;
        } else if piece.is_bishop() {
            self.all |= mask;
            self.pawns &= negative_mask;
            self.knights &= negative_mask;
            self.bishops |= mask;
            self.rooks &= negative_mask;
            self.queens &= negative_mask;
            self.kings &= negative_mask;
        } else if piece.is_rook() {
            self.all |= mask;
            self.pawns &= negative_mask;
            self.knights &= negative_mask;
            self.bishops &= negative_mask;
            self.rooks |= mask;
            self.queens &= negative_mask;
            self.kings &= negative_mask;
        } else if piece.is_queen() {
            self.all |= mask;
            self.pawns &= negative_mask;
            self.knights &= negative_mask;
            self.bishops &= negative_mask;
            self.rooks &= negative_mask;
            self.queens |= mask;
            self.kings &= negative_mask;
        } else if piece.is_king() {
            self.all |= mask;
            self.pawns &= negative_mask;
            self.knights &= negative_mask;
            self.bishops &= negative_mask;
            self.rooks &= negative_mask;
            self.queens &= negative_mask;
            self.kings |= mask;
        }
    }
    pub fn get_piece(&self, index: u8, team: Team) -> ChessPiece {
        let mask = 1 << index;
        if self.all & mask == 0 {
            return ChessPiece::None;
        }

        if self.kings & mask != 0 {
            return ChessPiece::King(team);
        }
        if self.queens & mask != 0 {
            return ChessPiece::Queen(team);
        }
        if self.rooks & mask != 0 {
            return ChessPiece::Rook(team);
        }
        if self.bishops & mask != 0 {
            return ChessPiece::Bishop(team);
        }
        if self.knights & mask != 0 {
            return ChessPiece::Knight(team);
        }
        if self.pawns & mask != 0 {
            return ChessPiece::Pawn(team);
        }
        panic!();
    }
    
    pub fn clear(&mut self) {
        self.all = 0;
        self.pawns = 0;
        self.knights = 0;
        self.bishops = 0;
        self.queens = 0;
        self.kings = 0;
        self.all = 0;
        self.pinned.fill(0);
    }
}

struct BoardIterator<T: Copy> {
    count: usize,
    current: usize,
    singles: [T; 64]
}
impl BoardIterator<usize> {
    fn index(board: u64) -> BoardIterator<usize> {
        let mut count: usize = 0;
        let mut singles: [usize; 64] = [0; 64];

        let top_16 = board as u16;
        let top_mid_16 = (board >> 16) as u16;
        let bottom_mid_16 = (board >> 32) as u16;
        let bottom_16 = (board >> 48) as u16;
        if top_16 != 0 {
            BoardIterator::divide_quad(&mut singles, &mut count, top_16, 0);
        }
        if top_mid_16 != 0 {
            BoardIterator::divide_quad(&mut singles, &mut count, top_mid_16, 16);
        }
        if bottom_mid_16 != 0 {
            BoardIterator::divide_quad(&mut singles, &mut count, bottom_mid_16, 32);
        }
        if bottom_16 != 0 {
            BoardIterator::divide_quad(&mut singles, &mut count, bottom_16, 48);
        }
        BoardIterator { count, singles, current: 0 }
    }
    fn divide_quad_quad(singles: &mut [usize; 64], count: &mut usize, quad_quad: u8, offset: usize) {
        if quad_quad & 0b0001 != 0 {
            singles[*count as usize] = offset;
            *count += 1;
        }
        if quad_quad & 0b0010 != 0 {
            singles[*count as usize] = offset + 1;
            *count += 1;
        }
        if quad_quad & 0b0100 != 0 {
            singles[*count as usize] = offset + 2;
            *count += 1;
        }
        if quad_quad & 0b1000 != 0 {
            singles[*count as usize] = offset + 3;
            *count += 1;
        }
    }
    fn divide_quad(singles: &mut [usize; 64], count: &mut usize, quad: u16, offset: usize) {
        let q0 = (quad & 0xF) as u8;
        let q1 = (quad >> 4 & 0xF) as u8;
        let q2 = (quad >> 8 & 0xF) as u8;
        let q3 = (quad >> 12 & 0xF) as u8;

        if q0 != 0 {
            Self::divide_quad_quad(singles, count, q0, offset);
        }
        if q1 != 0 {
            Self::divide_quad_quad(singles, count, q1, offset + 4);
        }
        if q2 != 0 {
            Self::divide_quad_quad(singles, count, q2, offset + 8);
        }
        if q3 != 0 {
            Self::divide_quad_quad(singles, count, q3, offset + 12);
        }
    }
}
impl BoardIterator<u64> {
    fn mask(board: u64) -> BoardIterator<u64> {
        let mut count: usize = 0;
        let mut singles: [u64; 64] = [0; 64];

        let top_16 = board as u16;
        let top_mid_16 = (board >> 16) as u16;
        let bottom_mid_16 = (board >> 32) as u16;
        let bottom_16 = (board >> 48) as u16;
        if top_16 != 0 {
            BoardIterator::divide_quad_mask(&mut singles, &mut count, top_16, 0);
        }
        if top_mid_16 != 0 {
            BoardIterator::divide_quad_mask(&mut singles, &mut count, top_mid_16, 16);
        }
        if bottom_mid_16 != 0 {
            BoardIterator::divide_quad_mask(&mut singles, &mut count, bottom_mid_16, 32);
        }
        if bottom_16 != 0 {
            BoardIterator::divide_quad_mask(&mut singles, &mut count, bottom_16, 48);
        }
        BoardIterator { count, singles, current: 0 }
    }
    fn divide_quad_quad_mask(singles: &mut [u64; 64], count: &mut usize, quad_quad: u8, offset: u8) {
        if quad_quad & 0b0001 != 0 {
            singles[*count as usize] = 1 << (offset);
            *count += 1;
        }
        if quad_quad & 0b0010 != 0 {
            singles[*count as usize] = 1 << (offset + 1);
            *count += 1;
        }
        if quad_quad & 0b0100 != 0 {
            singles[*count as usize] = 1 << (offset + 2);
            *count += 1;
        }
        if quad_quad & 0b1000 != 0 {
            singles[*count as usize] = 1 << (offset + 3);
            *count += 1;
        }
    }
    fn divide_quad_mask(singles: &mut [u64; 64], count: &mut usize, quad: u16, offset: u8) {
        let q0 = (quad & 0xF) as u8;
        let q1 = (quad >> 4 & 0xF) as u8;
        let q2 = (quad >> 8 & 0xF) as u8;
        let q3 = (quad >> 12 & 0xF) as u8;

        if q0 != 0 {
            Self::divide_quad_quad_mask(singles, count, q0, offset);
        }
        if q1 != 0 {
            Self::divide_quad_quad_mask(singles, count, q1, offset + 4);
        }
        if q2 != 0 {
            Self::divide_quad_quad_mask(singles, count, q2, offset + 8);
        }
        if q3 != 0 {
            Self::divide_quad_quad_mask(singles, count, q3, offset + 12);
        }
    }
}
impl<T: Copy> Iterator for BoardIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.count {
            let i = self.current as usize;
            self.current += 1;
            Some(self.singles[i])
        } else {
            None
        }
    }
}

pub struct ChessBoard {
    black: Pieces,
    white: Pieces,
    all: u64,

    active_team: Team,
    en_passant: u64,
    castle: CastleRights,

    half_clock: u16,
    full_count: u16,

    played_moves: Vec<PlayedMove>,
}



impl ChessBoard {
    pub fn new() -> ChessBoard {
        ChessBoard { 
            black: Pieces::new(), 
            white: Pieces::new(), 
            active_team: Team::White, 
            all: 0,
            en_passant: 0,
            castle: CastleRights::none(),
            half_clock: 0,
            full_count: 0,
            played_moves: vec![]
        }
    }
    pub fn load_initial_position(&mut self) {
        self.load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    }
    pub fn load_fen(&mut self, fen_string: &str) -> Result<(), &str> {
        // self.team_to_move = Team::White;
        // self.castling = CastleAvailability::none();
        // self.en_passant = None;
        // self.fullmoves_count = 0;
        // self.halfmove_clock = 0;
        // self.move_stack = vec![];
        let fen_string = fen_string.trim();
        let active_team : Team;
        let en_passant: u64;
        let castling: CastleRights;
        let pieces: [ChessPiece; 64];

        let regex = Regex::new(r"^(?<ranks>(?:[pnbrqkPNBRQK1-8]{1,8}\/){7}[pnbrqkPNBRQK1-8]{1,8})\s+(?<turn>b|w)\s+(?<castle>-|[K|Q|k|q]{1,4})\s+(?<en_passant>-|[a-h][36])\s+(?<half_moves>\d+)\s+(?<full_moves>\d+)$").unwrap();

        if !regex.is_match(fen_string) {
            return Err("Invalid FEN string");
        }
        let capture = regex.captures(fen_string).unwrap();
        
        let rank_string = capture.name("ranks").unwrap().as_str();
        let turn_string = capture.name("turn").unwrap().as_str();
        let castle_string = capture.name("castle").unwrap().as_str();
        let en_passant_string = capture.name("en_passant").unwrap().as_str();
        let half_moves_string = capture.name("half_moves").unwrap().as_str();
        let full_moves_string = capture.name("full_moves").unwrap().as_str();
        
        {
            let mut data = [ChessPiece::None; 64];
            let mut rank_strings: [&str; 8] = ["";8];

            let mut current_rank = 0;
            let mut current_pos = 0;
            for (i, byte) in rank_string.bytes().enumerate() {
                if byte == b'/' {
                    rank_strings[current_rank] = &rank_string[current_pos..i];
                    current_pos = i + 1;
                    current_rank += 1;
                }
            }
            rank_strings[current_rank] = &rank_string[current_pos..];

            for i in 0..8 {
                let rank_str = rank_strings[i];
                let rank_index = 7 - i;
                let rank = &mut data[rank_index * 8..rank_index * 8 + 8];
                let mut count: usize = 0;
                for c in rank_str.chars() {
                    let piece = ChessPiece::from_char(c);
                    if piece.is_none() {
                        let empty = c.to_digit(10);
                        if empty.is_none() {
                            return Err("Invalid character in rank");
                        }
                        let empty = empty.unwrap() as usize;
                        if empty == 0 || empty > 8 {
                            return Err("Invalid character in rank");
                        }
                        count += empty;
                    } else {
                        if count > 8 {
                            return Err("Invalid count in rank");
                        }
                        rank[count] = piece;
                        count += 1;
                    }
                }
                if count > 8 {
                    return Err("Invalid count in rank");
                }
            }

            pieces = data;
        }

        match turn_string {
            "w" => active_team = Team::White,
            "b" => active_team = Team::Black,
            _ => panic!("Invalid")
        }

        {
            let mut castle = CastleRights::none();
            for c in castle_string.chars() {
                match c {
                    'Q' => castle.set_queenside(Team::White, true),
                    'K' => castle.set_kingside(Team::White, true),
                    'q' => castle.set_queenside(Team::Black, true),
                    'k' => castle.set_kingside(Team::Black, true),
                    '-' => {
                        castle = CastleRights::none();
                        break;
                    }
                    _ => panic!("Invalid")
                }
            }
            castling = castle;
        }


        match en_passant_string {
            "-" => en_passant = 0,
            _ =>
                match index_from_string(en_passant_string) {
                    Some(i) => en_passant = 1 << i,
                    None => panic!("Invalid En Passant String"),
                }
        }

        let halfmoves = u16::from_str_radix(half_moves_string, 10);
        let fullmoves = u16::from_str_radix(full_moves_string, 10);

        if halfmoves.is_err() || fullmoves.is_err() {
            return Err("Invalid half or full moves");
        }


        self.clear();
        for i in 0..64 {
            self.set_piece(pieces[i], i as u8);
        }
        self.castle = castling;
        self.en_passant = en_passant;
        self.active_team = active_team;
        self.half_clock = halfmoves.unwrap();
        self.full_count = fullmoves.unwrap();


        Ok(())
    }

    pub fn to_fen(&self) -> String {
        let mut rank_strings = vec![];
        let data = self.piece_array();
        for i in 0..8 {
            let mut str = String::new();

            let mut empty = 0;
            for j in 0..8 {
                let piece = data[i * 8 + j];
                if piece.is_none() {
                    empty += 1;
                } else {
                    if empty > 0 {
                        str = str + &empty.to_string();
                        empty = 0;
                    }
                    str = str + &piece.char().to_string();
                }
            }
            if empty > 0 {
                str = str + &empty.to_string();
            }

            rank_strings.push(str);
        }
        let mut fen_string = String::new();
        for i in (0..8).rev() {
            fen_string = fen_string + &rank_strings[i];
            if i != 0 {
                fen_string += "/";
            }
        }
        fen_string += " ";
        fen_string += match self.active_team {
            Team::White => "w",
            Team::Black => "b",
        };
        fen_string += " ";
        fen_string += &self.castle.as_string();

        fen_string += " ";

        if self.en_passant != 0 {
            let file = self.en_passant.trailing_zeros() % 8;
            let rank = self.en_passant.trailing_zeros() / 8;
            fen_string.push_str(&format!("{}{}",file_char(file as i8),rank_char(rank as i8)));
        } else {
            fen_string += "-";
        }

        fen_string += " ";
        fen_string += &format!("{} {}", self.half_clock, self.full_count);

        fen_string
    }

    pub fn clear(&mut self) {
        self.all = 0;
        self.white.clear();
        self.black.clear();
    }
    pub fn set_piece(&mut self, piece: ChessPiece, index: u8) {
        let mask = 1 << index;
        let negative_mask = !mask;
        if piece.is_none() {
            self.all &= negative_mask;
            self.white.set_piece(piece, index);
            self.black.set_piece(piece, index);
        } else if piece.is_team(Team::White) {
            self.all |= mask;
            self.white.set_piece(piece, index);
            self.black.set_piece(ChessPiece::None, index);
        } else {
            self.all |= mask;
            self.black.set_piece(piece, index);
            self.white.set_piece(ChessPiece::None, index);
        }
    }
    pub fn get_piece(&self, index: u8) -> ChessPiece {
        let mask = 1 << index;
        if self.all & mask == 0 {
            return ChessPiece::None;
        }
        
        if self.white.all & mask != 0 {
            return self.white.get_piece(index, Team::White);
        } else {
            return self.black.get_piece(index, Team::Black);
        }
    }

    fn team_pieces(&self, team: Team) -> &Pieces {
        match team {
            Team::White => &self.white,
            Team::Black => &self.black,
        }
    }

    pub fn play_move(&mut self, chess_move: ChessMove) -> &PlayedMove {
        
        let chess_piece = self.get_piece(chess_move.from);

        let previous_en_passant = self.en_passant;
        let previous_castle_rights = self.castle;
        let captured_piece: ChessPiece;

        let mut half_move = true;

        match chess_move.move_type {
            super::moves::MoveType::Move => {
                captured_piece = self.get_piece(chess_move.to);
                self.en_passant = 0; // reset enpassant on a move that isnt en passant

                self.set_piece(chess_piece, chess_move.to);
                self.set_piece(ChessPiece::None, chess_move.from);

                if chess_piece.is_king() {
                    self.castle.set_team(chess_piece.team(), false);
                }
                if chess_move.from == 0 || chess_move.to == 0 {
                    self.castle.set_queenside(Team::White, false);
                } 
                if chess_move.from == 7 || chess_move.to == 7 {
                    self.castle.set_kingside(Team::White, false);
                } 
                if chess_move.from == 56 || chess_move.to == 56 {
                    self.castle.set_queenside(Team::Black, false);
                } 
                if chess_move.from == 63 || chess_move.to == 63 {
                    self.castle.set_kingside(Team::Black, false);
                } 
            },
            super::moves::MoveType::Pawn(en_passant_square) => {
                captured_piece = self.get_piece(chess_move.to);
                if let Some(en_passant_square) = en_passant_square {
                    self.en_passant = 1 << en_passant_square;
                } else {
                    self.en_passant = 0;
                }
                half_move = false;

                self.set_piece(chess_piece, chess_move.to);
                self.set_piece(ChessPiece::None, chess_move.from);

                if chess_move.from == 0 || chess_move.to == 0 {
                    self.castle.set_queenside(Team::White, false);
                } 
                if chess_move.from == 7 || chess_move.to == 7 {
                    self.castle.set_kingside(Team::White, false);
                } 
                if chess_move.from == 56 || chess_move.to == 56 {
                    self.castle.set_queenside(Team::Black, false);
                } 
                if chess_move.from == 63 || chess_move.to == 63 {
                    self.castle.set_kingside(Team::Black, false);
                } 
            },
            super::moves::MoveType::EnPassant(en_passant_capture) => {
                captured_piece = self.get_piece(en_passant_capture);
                self.en_passant = 0;

                self.set_piece(chess_piece, chess_move.to);
                self.set_piece(ChessPiece::None, chess_move.from);
                self.set_piece(ChessPiece::None, en_passant_capture);
            },
            super::moves::MoveType::Promotion(promotion_type) => {
                captured_piece = self.get_piece(chess_move.to);
                self.en_passant = 0;

                self.set_piece(promotion_type, chess_move.to);
                self.set_piece(ChessPiece::None, chess_move.from);

                if chess_move.from == 0 || chess_move.to == 0 {
                    self.castle.set_queenside(Team::White, false);
                } 
                if chess_move.from == 7 || chess_move.to == 7 {
                    self.castle.set_kingside(Team::White, false);
                } 
                if chess_move.from == 56 || chess_move.to == 56 {
                    self.castle.set_queenside(Team::Black, false);
                } 
                if chess_move.from == 63 || chess_move.to == 63 {
                    self.castle.set_kingside(Team::Black, false);
                } 
            },
            super::moves::MoveType::Castle(castle_type) => {
                captured_piece = ChessPiece::None;
                self.en_passant = 0;

                let from_king = castle_type.get_king_index();
                let to_king = castle_type.get_new_king_index();
                let from_rook = castle_type.get_rook_index();
                let to_rook = castle_type.get_new_rook_index();

                let king = self.get_piece(from_king);
                let rook = self.get_piece(from_rook);
                self.set_piece(king, to_king);
                self.set_piece(rook, to_rook);
                self.set_piece(ChessPiece::None, from_king);
                self.set_piece(ChessPiece::None, from_rook);

                //remove castle rights from team when castled
                self.castle.set_team(king.team(), false);
            },
        };

        let played = 
        PlayedMove::new(
            chess_move, 
            chess_piece, 
            captured_piece, 
            previous_castle_rights,
            previous_en_passant
        );

        //Switch team
        self.active_team = self.active_team.other();
        if matches!(self.active_team, Team::White) {
            self.full_count += 1;
        }
        half_move = half_move || !captured_piece.is_none();
        if half_move {
            //self.half_clock += 1;
        }
        
        self.played_moves.push(played);
        self.played_moves.last().unwrap()
    }
    pub fn undo_move(&mut self) -> Option<PlayedMove> {
        let to_undo = self.played_moves.pop();

        if let Some(played) = &to_undo {
            self.active_team = self.active_team.other();
            if matches!(self.active_team, Team::Black) {
                self.full_count -= 1;
            }

            let original_move = &played.original;
            let chess_piece = played.piece;
            let captured_piece = played.captured;

            let previous_en_passant = played.previous_en_passant;
            let previous_castle_rights = played.previous_castle_rights;

            self.en_passant = previous_en_passant;
            self.castle = previous_castle_rights;

            match original_move.move_type {
                super::moves::MoveType::Move => {
                    self.set_piece(chess_piece, original_move.from);
                    self.set_piece(captured_piece, original_move.to);
                },
                super::moves::MoveType::Pawn(_) => {
                    self.set_piece(chess_piece, original_move.from);
                    self.set_piece(captured_piece, original_move.to);
                },
                super::moves::MoveType::EnPassant(en_passant_capture) => {
                    self.set_piece(chess_piece, original_move.from);
                    self.set_piece(ChessPiece::None, original_move.to);
                    self.set_piece(captured_piece, en_passant_capture);
                },
                super::moves::MoveType::Promotion(_) => {
                    self.set_piece(chess_piece, original_move.from);
                    self.set_piece(captured_piece, original_move.to);
                },
                super::moves::MoveType::Castle(castle_type) => {
                    let from_king = castle_type.get_king_index();
                    let to_king = castle_type.get_new_king_index();
                    let from_rook = castle_type.get_rook_index();
                    let to_rook = castle_type.get_new_rook_index();

                    let king = self.get_piece(to_king);
                    let rook = self.get_piece(to_rook);
                    self.set_piece(king, from_king);
                    self.set_piece(rook, from_rook);
                    self.set_piece(ChessPiece::None, to_king);
                    self.set_piece(ChessPiece::None, to_rook);
                },
            };
        }

        to_undo
    }


    fn shift_team_pawn(&self, pawns: u64, shift: u8) -> u64 {
        match self.active_team {
            Team::White => {
                pawns << shift
            },
            Team::Black => {
                pawns >> (16 - shift)
            },
        }
    }
    fn is_pawn_unmoved(&self, pawn: u64) -> bool {
        match self.active_team {
            Team::White => pawn & RANK_2 != 0,
            Team::Black => pawn & RANK_7 != 0,
        }
    }
    fn is_promotion_square(&self, pawn: u64) -> bool {
        match self.active_team {
            Team::White => pawn & RANK_8 != 0,
            Team::Black => pawn & RANK_1 != 0,
        }
    }

    fn generate_pawn_moves(&self, moves: &mut Vec<ChessMove>, move_mask: u64, pin_masks: &[u64; 64]) {
        let pawns = self.team_pieces(self.active_team).pawns;

        fn add_promotion(board: &ChessBoard, moves: &mut Vec<ChessMove>, pawn: u64, from: u8, to: u8) -> bool {
            if board.is_promotion_square(pawn) {
                moves.push(ChessMove::new(from, to, MoveType::Promotion(ChessPiece::Bishop(board.active_team))));
                moves.push(ChessMove::new(from, to, MoveType::Promotion(ChessPiece::Knight(board.active_team))));
                moves.push(ChessMove::new(from, to, MoveType::Promotion(ChessPiece::Rook(board.active_team))));
                moves.push(ChessMove::new(from, to, MoveType::Promotion(ChessPiece::Queen(board.active_team))));
                return true;
            }
            false
        }

        for i in BoardIterator::index(pawns) {
            let pawn_bit = 1 << i;
            let push_one = self.shift_team_pawn(pawn_bit, 8) & !self.all;
            
            if push_one != 0 {
                let move_mask = pin_masks[i] & move_mask;
                {
                    let push_one = push_one & move_mask;
                    if push_one != 0 {
                        if !add_promotion(self, moves, push_one, i as u8, push_one.trailing_zeros() as u8) {
                            moves.push(ChessMove::new(i as u8, push_one.trailing_zeros() as u8, MoveType::Pawn(None)));
                        }
                    }
                }
                let push_two = self.shift_team_pawn(push_one, 8) & !self.all & move_mask;
                if push_two != 0 && self.is_pawn_unmoved(pawn_bit) {
                    moves.push(ChessMove::new(i as u8, push_two.trailing_zeros() as u8, MoveType::Pawn(Some(push_one.trailing_zeros() as u8))));
                }
            }

            let capture_mask = self.team_pieces(self.active_team.other()).all | self.en_passant;
            let move_mask = move_mask | self.shift_team_pawn(move_mask, 8) & self.en_passant;
            let move_mask = pin_masks[i] & move_mask;
            
            let capture_right = self.shift_team_pawn(pawn_bit & !FILE_A, 7) & capture_mask & move_mask;
            if capture_right != 0 {
                let capture_right_index = capture_right.trailing_zeros() as u8;
                if !add_promotion(self, moves, capture_right, i as u8, capture_right_index) {
                    let en_passant = capture_right & self.en_passant != 0;
                    if en_passant {
                        
                        moves.push(ChessMove::new(i as u8, capture_right_index, MoveType::EnPassant((i - 1) as u8)));
                    } else {
                        moves.push(ChessMove::new(i as u8, capture_right_index, MoveType::Move));
                    }
                }
            }
            let capture_left = self.shift_team_pawn(pawn_bit & !FILE_H, 9) & capture_mask & move_mask;
            if capture_left != 0 {
                let capture_index = capture_left.trailing_zeros() as u8;
                if !add_promotion(self, moves, capture_left, i as u8, capture_index) {
                    let en_passant = capture_left & self.en_passant != 0;
                    if en_passant {
                        moves.push(ChessMove::new(i as u8, capture_index, MoveType::EnPassant((i + 1) as u8)));
                    } else {
                        moves.push(ChessMove::new(i as u8, capture_index, MoveType::Move));
                    }
                    
                }
            }
        }
    }
    
    fn generate_knight_moves(&self, moves: &mut Vec<ChessMove>, move_mask: u64, pin_masks: &[u64; 64]) {
        let pieces = self.team_pieces(self.active_team);
        let knights = pieces.knights;
        let all = pieces.all;
        let spots = !all & move_mask;

        let ab_mask = !(FILE_A | FILE_B);
        let gh_mask = !(FILE_G | FILE_H);

        fn add_move(moves: &mut Vec<ChessMove>, from: u8, to: u64) {
            if to == 0 {
                return;
            }
            moves.push(ChessMove::new(from, to.trailing_zeros() as u8, MoveType::Move));
        }

        for i in BoardIterator::index(knights) {
            let spots = spots & pin_masks[i];
            let knight_bit = 1 << i;

            add_move(moves, i as u8, (knight_bit & !FILE_A) << 15 & spots);
            add_move(moves, i as u8, (knight_bit & !FILE_A) >> 17 & spots);
            add_move(moves, i as u8, (knight_bit & !FILE_H) << 17 & spots);
            add_move(moves, i as u8, (knight_bit & !FILE_H) >> 15 & spots);
            add_move(moves, i as u8, (knight_bit & ab_mask) << 6 & spots);
            add_move(moves, i as u8, (knight_bit & ab_mask) >> 10 & spots);
            add_move(moves, i as u8, (knight_bit & gh_mask) << 10 & spots);
            add_move(moves, i as u8, (knight_bit & gh_mask) >> 6 & spots);
        }
    }

    fn generate_ray_move_left(&self, moves: &mut Vec<ChessMove>, origin: u8, shift: u8, edge_mask: u64, team: u64, opponent: u64, move_mask: u64) {
        let mut moved = 1 << origin;
        for _ in 1..=7 {
            moved &= edge_mask & !opponent;

            moved = (moved << shift) & !team;

            if moved == 0 {
                break;
            } else if moved & move_mask != 0 {
                moves.push(ChessMove::new(origin, moved.trailing_zeros() as u8, MoveType::Move));
            }
        }
    }
    fn generate_ray_move_right(&self, moves: &mut Vec<ChessMove>, origin: u8, shift: u8, edge_mask: u64, team: u64, opponent: u64, move_mask: u64) {
        let mut moved = 1 << origin;
        for _ in 1..=7 {
            moved &= edge_mask & !opponent;

            moved = (moved >> shift) & !team;

            if moved == 0 {
                break;
            } else if moved & move_mask != 0 {
                moves.push(ChessMove::new(origin, moved.trailing_zeros() as u8, MoveType::Move));
            }
        }
    }
    
    fn generate_rook_moves(&self, moves: &mut Vec<ChessMove>, move_mask: u64, pin_masks: &[u64; 64]) {
        let pieces = self.team_pieces(self.active_team);
        let rooks = pieces.rooks;
        let team = pieces.all;
        let enemy = self.team_pieces(self.active_team.other()).all;

        for i in BoardIterator::index(rooks) {
            let move_mask = move_mask & pin_masks[i];
            self.generate_ray_move_left(moves, i as u8, 1, !FILE_H, team, enemy, move_mask);
            self.generate_ray_move_right(moves, i as u8, 1, !FILE_A, team, enemy, move_mask);
            self.generate_ray_move_left(moves, i as u8, 8, !0, team, enemy, move_mask);
            self.generate_ray_move_right(moves, i as u8, 8, !0, team, enemy, move_mask);
        }
    }
    fn generate_bishop_moves(&self, moves: &mut Vec<ChessMove>, move_mask: u64, pin_masks: &[u64; 64]) {
        let pieces = self.team_pieces(self.active_team);
        let bishops = pieces.bishops;
        let team = pieces.all;
        let enemy = self.team_pieces(self.active_team.other()).all;

        for i in BoardIterator::index(bishops) {
            let move_mask = move_mask & pin_masks[i];
            self.generate_ray_move_left(moves, i as u8, 9, !FILE_H, team, enemy, move_mask);
            self.generate_ray_move_right(moves, i as u8, 7, !FILE_H, team, enemy, move_mask);
            self.generate_ray_move_left(moves, i as u8, 7, !FILE_A, team, enemy, move_mask);
            self.generate_ray_move_right(moves, i as u8, 9, !FILE_A, team, enemy, move_mask);
        }
    }
    fn generate_queen_moves(&self, moves: &mut Vec<ChessMove>, move_mask: u64, pin_masks: &[u64; 64]) {
        let pieces = self.team_pieces(self.active_team);
        let queens = pieces.queens;
        let team = pieces.all;
        let enemy = self.team_pieces(self.active_team.other()).all;

        for i in BoardIterator::index(queens) {
            let move_mask = move_mask & pin_masks[i];
            self.generate_ray_move_left(moves, i as u8, 9, !FILE_H, team, enemy, move_mask);
            self.generate_ray_move_right(moves, i as u8, 7, !FILE_H, team, enemy, move_mask);
            self.generate_ray_move_left(moves, i as u8, 7, !FILE_A, team, enemy, move_mask);
            self.generate_ray_move_right(moves, i as u8, 9, !FILE_A, team, enemy, move_mask);
            self.generate_ray_move_left(moves, i as u8, 1, !FILE_H, team, enemy, move_mask);
            self.generate_ray_move_right(moves, i as u8, 1, !FILE_A, team, enemy, move_mask);
            self.generate_ray_move_left(moves, i as u8, 8, !0, team, enemy, move_mask);
            self.generate_ray_move_right(moves, i as u8, 8, !0, team, enemy, move_mask);
        }
    }
    fn generate_king_moves(&self, moves: &mut Vec<ChessMove>, threats: u64) {
        let pieces = self.team_pieces(self.active_team);
        let kings = pieces.kings;
        let spots = !pieces.all & !threats;


        if threats & kings == 0 {
            if self.castle.get_kingside(self.active_team) {
                let mask = match self.active_team {
                    Team::White => castle::WHITE_KINGSIDE_EMPTY_MAKS,
                    Team::Black => castle::BLACK_KINGSIDE_EMPTY_MAKS,
                };
                if self.all & mask == 0 && threats & mask == 0 {
                    moves.push(ChessMove::new(0, 0, MoveType::Castle(CastleType::kingside_of(self.active_team))))
                }
            }
            if self.castle.get_queenside(self.active_team) {
                let mask = match self.active_team {
                    Team::White => castle::WHITE_QUEENSIDE_EMPTY_MAKS,
                    Team::Black => castle::BLACK_QUEENSIDE_EMPTY_MAKS,
                };
                let threat_mask = match self.active_team {
                    Team::White => castle::WHITE_QUEENSIDE_THREAT_MASK,
                    Team::Black => castle::BLACK_QUEENSIDE_THREAT_MASK,
                };
                if self.all & mask == 0 && threats & threat_mask == 0 {
                    moves.push(ChessMove::new(0, 0, MoveType::Castle(CastleType::queenside_of(self.active_team))))
                }
            }
        }

        fn add_move(moves: &mut Vec<ChessMove>, from: u8, to: u64) {
            if to == 0 {
                return;
            }
            moves.push(ChessMove::new(from, to.trailing_zeros() as u8, MoveType::Move));
        }

        for i in BoardIterator::index(kings) {
            let king_bit = 1 << i;

            let king_moves = Self::king_threats(king_bit) & spots;
            for j in BoardIterator::mask(king_moves) {
                add_move(moves, i as u8, j);
            }

            
            // add_move(moves, i as u8, (king_bit & !FILE_A) >> 1 & spots);
            // add_move(moves, i as u8, (king_bit & !FILE_A) >> 9 & spots);
            // add_move(moves, i as u8, (king_bit & !FILE_H) >> 1 & spots);
            // add_move(moves, i as u8, (king_bit & !FILE_H) >> 9 & spots);
            // add_move(moves, i as u8, (king_bit & !FILE_H) << 7 & spots);
            // add_move(moves, i as u8, (king_bit) << 8 & spots);
            // add_move(moves, i as u8, (king_bit) >> 8 & spots);
        }
    }
    

    fn pawn_threats(pawns: u64, team: Team) -> u64 {
        match team {
            Team::White => (pawns & !FILE_H) << 9 | (pawns & !FILE_A) << 7,
            Team::Black => (pawns & !FILE_H) >> 7 | (pawns & !FILE_A) >> 9,
        } 
    }
    fn knight_threats(knights: u64) -> u64 {
        let ab_mask = !(FILE_A | FILE_B);
        let gh_mask = !(FILE_G | FILE_H);

        (knights & !FILE_A) << 15 |
        (knights & !FILE_A) >> 17 |
        (knights & !FILE_H) << 17 |
        (knights & !FILE_H) >> 15 |
        (knights & ab_mask) << 6 |
        (knights & ab_mask) >> 10 |
        (knights & gh_mask) << 10 |
        (knights & gh_mask) >> 6
    }
    fn rook_threats(rooks: u64, all: u64) -> u64 {
        let mut threats = 0;

        let mut up = rooks;
        let mut down = rooks;
        let mut left = rooks;
        let mut right = rooks;
        for _ in 1..=7 {
            left &= !FILE_A;
            right &= !FILE_H;
            

            up >>= 8;
            down <<= 8;
            left >>= 1;
            right <<= 1;

            threats |= up | down | left | right;

            left &= !all;
            right &= !all;
            up &= !all;
            down &= !all;
        }
        threats
    }
    fn bishop_threats(bishops: u64, all: u64) -> u64 {
        let mut threats = 0;

        let mut up_left = bishops;
        let mut up_right = bishops;
        let mut down_left = bishops;
        let mut down_right = bishops;
        for _ in 1..=7 {
            up_left &= !FILE_A;
            down_left &= !FILE_A;
            up_right &= !FILE_H;
            down_right &= !FILE_H;
            
            up_left >>= 9;
            down_left <<= 7;
            up_right >>= 7;
            down_right <<= 9;

            threats |= up_left | down_left | up_right | down_right;

            up_left &= !all;
            down_left &= !all;
            up_right &= !all;
            down_right &= !all;
        }
        threats
    }
    fn queen_threats(queens: u64, all: u64) -> u64 {
        let mut threats = 0;

        let mut up = queens;
        let mut down = queens;
        let mut left = queens;
        let mut right = queens;
        let mut up_left = queens;
        let mut up_right = queens;
        let mut down_left = queens;
        let mut down_right = queens;
        for _ in 1..=7 {
            left &= !FILE_A;
            right &= !FILE_H;
            up_left &= !FILE_A;
            down_left &= !FILE_A;
            up_right &= !FILE_H;
            down_right &= !FILE_H;
            
            up >>= 8;
            down <<= 8;
            left >>= 1;
            right <<= 1;
            up_left >>= 9;
            down_left <<= 7;
            up_right >>= 7;
            down_right <<= 9;

            threats |= up | down | left | right;
            threats |= up_left | down_left | up_right | down_right;

            left &= !all;
            right &= !all;
            up &= !all;
            down &= !all;
            up_left &= !all;
            down_left &= !all;
            up_right &= !all;
            down_right &= !all;
        }
        threats
    }
    fn king_threats(kings: u64) -> u64 {
        (kings & !FILE_A) << 7 |
        (kings & !FILE_A) >> 1 |
        (kings & !FILE_A) >> 9 |
        (kings & !FILE_H) << 1 |
        (kings & !FILE_H) << 9 |
        (kings & !FILE_H) >> 7 |
        (kings) << 8 |
        (kings) >> 8
    }
    fn threats_to_king(&self, team: Team) -> u64 {
        let pieces = self.team_pieces(team);
        let oposing = self.team_pieces(team.other());
        let all_minus_king = self.all & !pieces.kings;

        Self::pawn_threats(oposing.pawns, team.other()) |
        Self::knight_threats(oposing.knights) |
        Self::bishop_threats(oposing.bishops, all_minus_king) |
        Self::rook_threats(oposing.rooks, all_minus_king) |
        Self::queen_threats(oposing.queens, all_minus_king) |
        Self::king_threats(oposing.kings)
    }


    fn get_savior_mask(&self, target: u64, team: Team, pin_masks: &mut [u64; 64]) -> u64 {
        let oposing = self.team_pieces(team.other());
        let own = self.team_pieces(team);
        
        let pawn_knights = 
        Self::pawn_threats(target, team) & oposing.pawns | // gets diagonal pieces in front of the piece
        Self::knight_threats(target) & oposing.knights;

        let en_passant_pawn = match team {
            Team::Black => (RANK_3 & self.en_passant) << 8 & oposing.pawns,
            Team::White => (RANK_6 & self.en_passant) >> 8 & oposing.pawns,
        };
        let oposing_minus_enpassant_pawn = oposing.all & !en_passant_pawn;

        let rooks = oposing.rooks | oposing.queens;
        let bishops = oposing.bishops | oposing.queens;

        let mut up = target;
        let mut down = target;
        let mut left = target;
        let mut right = target;
        let mut up_left = target;
        let mut up_right = target;
        let mut down_left = target;
        let mut down_right = target;
        for _ in 1..=7 {
            left &= !FILE_A;
            right &= !FILE_H;
            up_left &= !FILE_A;
            down_left &= !FILE_A;
            up_right &= !FILE_H;
            down_right &= !FILE_H;

            left &= !oposing_minus_enpassant_pawn;
            right &= !oposing_minus_enpassant_pawn;
            up &= !oposing.all;
            down &= !oposing.all;
            up_left &= !oposing.all;
            down_left &= !oposing.all;
            up_right &= !oposing.all;
            down_right &= !oposing.all;
            
            up |= up >> 8;
            down |= down << 8;
            left |= left >> 1;
            right |= right << 1;
            up_left |= up_left >> 9;
            down_left |= down_left << 7;
            up_right |= up_right >> 7;
            down_right |= down_right << 9;
        }
        up &= !target;
        down &= !target;
        left &= !target;
        right &= !target;
        up_left &= !target;
        up_right &= !target;
        down_left &= !target;
        down_right &= !target;

        let mut capture_squares = pawn_knights;

        fn test_ray(ray: u64, attackers: u64, own: u64, capture_squares: &mut u64, pin_masks: &mut [u64; 64]) {
            //println!("{} {}", ray, attackers);
            if ray & attackers != 0 {
                let pinned = ray & own;
                if pinned == 0 {
                    *capture_squares |= ray;
                } else if pinned.count_ones() == 1 {
                    pin_masks[pinned.trailing_zeros() as usize] = ray;
                }
            }
        }
        fn test_ray_with_ep(ray: u64, attackers: u64, own: u64, own_pawns: u64, en_passant: u64, team: Team, capture_squares: &mut u64, pin_masks: &mut [u64; 64]) {
            if ray & attackers != 0 {
                let pinned = ray & own;
                if pinned == 0 {
                    *capture_squares |= ray;
                } else if pinned.count_ones() == 1 {
                    if en_passant != 0 && pinned & own_pawns != 0  {
                        if ChessBoard::pawn_threats(pinned, team) & en_passant != 0 {
                            pin_masks[pinned.trailing_zeros() as usize] = !en_passant;
                        }
                    } else {
                        pin_masks[pinned.trailing_zeros() as usize] = ray;
                    }
                }
            }
        }

        test_ray_with_ep(left, rooks, own.all, own.pawns, self.en_passant, team, &mut capture_squares, pin_masks);
        test_ray_with_ep(right, rooks, own.all, own.pawns, self.en_passant, team, &mut capture_squares, pin_masks);
        test_ray(up, rooks, own.all, &mut capture_squares, pin_masks);
        test_ray(down, rooks, own.all, &mut capture_squares, pin_masks);
        test_ray(up_left, bishops, own.all, &mut capture_squares, pin_masks);
        test_ray(up_right, bishops, own.all, &mut capture_squares, pin_masks);
        test_ray(down_left, bishops, own.all, &mut capture_squares, pin_masks);
        test_ray(down_right, bishops, own.all, &mut capture_squares, pin_masks);

        let attackers = capture_squares & oposing.all;
        if attackers.count_ones() > 1 {
            // The king must move to beable to escape
            return 0;
        }
        capture_squares
    }

    fn capture_mask(&self) -> u64 {
        let pieces = self.team_pieces(self.active_team);
        let king_bit = pieces.kings;

        let mut pin_mask = [!0u64; 64];

        let mut capture_mask = self.get_savior_mask(king_bit, self.active_team, &mut pin_mask);
        capture_mask
    }
    pub fn generate_moves(&self) -> Vec<ChessMove> {
        let mut moves = vec![];

        let threats = self.threats_to_king(self.active_team);
        let pieces = self.team_pieces(self.active_team);

        let king_bit = pieces.kings;

        let mut pin_mask = [!0u64; 64];

        let mut capture_mask = self.get_savior_mask(king_bit, self.active_team, &mut pin_mask);

        if king_bit & threats == 0 {
            capture_mask = !0;
        }

        {
            let moves = &mut moves;
            if capture_mask != 0 {
                self.generate_pawn_moves(moves, capture_mask, &pin_mask);
                self.generate_knight_moves(moves, capture_mask, &pin_mask);
                self.generate_rook_moves(moves, capture_mask, &pin_mask);
                self.generate_bishop_moves(moves, capture_mask, &pin_mask);
                self.generate_queen_moves(moves, capture_mask, &pin_mask);
            }
            self.generate_king_moves(moves, threats);
        }
        
        moves
    }

    pub fn piece_array(&self) -> [ChessPiece; 64] {
        let mut arr = [ChessPiece::None; 64];
        
        for i in BoardIterator::index(self.all) {
            let mask = 1 << i;
            if self.white.kings & mask != 0 {
                arr[i] = ChessPiece::King(Team::White);
            } else if self.white.queens & mask != 0 {
                arr[i] = ChessPiece::Queen(Team::White);
            } else if self.white.bishops & mask != 0 {
                arr[i] = ChessPiece::Bishop(Team::White);
            } else if self.white.rooks & mask != 0 {
                arr[i] = ChessPiece::Rook(Team::White);
            } else if self.white.knights & mask != 0 {
                arr[i] = ChessPiece::Knight(Team::White);
            } else if self.white.pawns & mask != 0 {
                arr[i] = ChessPiece::Pawn(Team::White);
            } else if self.black.kings & mask != 0 {
                arr[i] = ChessPiece::King(Team::Black);
            } else if self.black.queens & mask != 0 {
                arr[i] = ChessPiece::Queen(Team::Black);
            } else if self.black.bishops & mask != 0 {
                arr[i] = ChessPiece::Bishop(Team::Black);
            } else if self.black.rooks & mask != 0 {
                arr[i] = ChessPiece::Rook(Team::Black);
            } else if self.black.knights & mask != 0 {
                arr[i] = ChessPiece::Knight(Team::Black);
            } else if self.black.pawns & mask != 0 {
                arr[i] = ChessPiece::Pawn(Team::Black);
            }
        }

        arr
    }
}

impl Display for ChessBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        let pieces = self.piece_array();
        let ranks = (0..8).enumerate();
        let files = (0..8).enumerate();
        let files = Vec::from_iter(files);


        let offset = " ";

        let file_on_bottom = false;

        let checks = self.threats_to_king(Team::Black) & self.black.kings | self.threats_to_king(Team::White) & self.white.kings;
        let threats = self.threats_to_king(self.active_team);
        let threats = self.capture_mask();


        let mut file_string = format!("{}  ", offset);
        for (_, file) in &files {
            file_string.push_str(&format!("  {} ", file_char(*file)));
        }
        file_string.push('\n');

        

        write!(f, "{}  {:^11}{:^11}{:^11}\n", offset, "", "Chess","").unwrap();
        if !file_on_bottom {
            write!(f, "{}", file_string)?;
        }
        write!(f, "{}  ┌───┬───┬───┬───┬───┬───┬───┬───┐\n", offset)?;
        for (i, y) in ranks {
            write!(f, "{}{} ", offset, rank_char(y))?;

            for (_, x) in &files {
                let index = y * 8 + x;
                //let check = checks & 1 << index != 0;
                let check = threats & 1<< index != 0;
                let check = if check {"!".red()} else {" ".stylize()};
                write!(f, "│{}{} ", check, pieces[index as usize])?
            }
            write!(f, "│\n")?;
            if i < 8 - 1 {
                write!(f, "{}  ├───┼───┼───┼───┼───┼───┼───┼───┤\n", offset)?;
            } else {
                write!(f, "{}  └───┴───┴───┴───┴───┴───┴───┴───┘\n", offset)?;
            }
        }
        if file_on_bottom {
            write!(f, "{}", file_string)?;
        }
        

        Ok(())
    }
}