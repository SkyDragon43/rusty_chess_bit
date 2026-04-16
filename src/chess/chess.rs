use std::{arch::x86_64::_MM_FROUND_TO_ZERO, collections::VecDeque, error::Error, fmt::Display, hint::black_box, path::Iter, string::ParseError};

use regex::{Regex};

use crate::{castle::CastleRights, chess::{constants::{ANTI_DIAGONAL_0, ANTI_DIAGONALS, DIAGONAL_0, DIAGONALS, file_char, get_anti_diagonal, get_diagonal, index_from_string, rank_char}, piece::{ChessPiece, Team}}, moves::{ChessMove, PlayedMove}};



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

    pub fn play_move(&mut self, chess_move: ChessMove) -> &PlayedMove {
        
        let chess_piece = self.get_piece(chess_move.from);

        let previous_en_passant = self.en_passant;
        let previous_castle_rights = self.castle;
        let captured_piece: ChessPiece;

        match chess_move.move_type {
            super::moves::MoveType::Move => {
                captured_piece = self.get_piece(chess_move.to);
                self.en_passant = 0; // reset enpassant on a move that isnt en passant

                self.set_piece(chess_piece, chess_move.to);
                self.set_piece(ChessPiece::None, chess_move.from);
            },
            super::moves::MoveType::Pawn(en_passant_square) => {
                captured_piece = self.get_piece(chess_move.to);
                if let Some(en_passant_square) = en_passant_square {
                    self.en_passant = 1 << en_passant_square;
                } else {
                    self.en_passant = 0;
                }

                self.set_piece(chess_piece, chess_move.to);
                self.set_piece(ChessPiece::None, chess_move.from);
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
            },
            super::moves::MoveType::Castle(castle_type) => {
                captured_piece = ChessPiece::None;

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
        
        self.played_moves.push(played);
        self.played_moves.last().unwrap()
    }
    pub fn undo_move(&mut self) -> Option<PlayedMove> {
        let to_undo = self.played_moves.pop();

        if let Some(played) = &to_undo {
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
                write!(f, "│ {} ", pieces[index as usize])?
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