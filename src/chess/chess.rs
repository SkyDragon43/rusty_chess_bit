use std::{error::Error, fmt::Display, hint::black_box, string::ParseError};

use regex::{Regex};

use crate::chess::{constants::{ANTI_DIAGONAL_0, ANTI_DIAGONALS, DIAGONAL_0, DIAGONALS, file_char, get_anti_diagonal, get_diagonal, index_from_string, rank_char}, piece::{ChessPiece, Team}};



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
pub struct ChessBoard {
    black: Pieces,
    white: Pieces,
    all: u64,

    active_team: Team,
    en_passant: u64,
    castle: u64,

    half_clock: u16,
    full_count: u16
}



impl ChessBoard {
    pub fn new() -> ChessBoard {
        ChessBoard { 
            black: Pieces::new(), 
            white: Pieces::new(), 
            active_team: Team::White, 
            all: 0,
            en_passant: 0,
            castle: 0,
            half_clock: 0,
            full_count: 0
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

        //println!("RANK IS: '{}'", rank_string);

        //self.data = [ChessPiece::from_id(0); BOARD_WIDTH * BOARD_HEIGHT];
        self.clear();
        
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

            for i in 0..64 {
                self.set_piece(data[i], i as u8);
            }
        }

        match turn_string {
            "w" => self.active_team = Team::White,
            "b" => self.active_team = Team::Black,
            _ => panic!("Invalid")
        }

        // for c in castle_string.chars() {
        //     match c {
        //         'Q' => self.castle.set_queenside(Team::White, true),
        //         'K' => self.castle.set_kingside(Team::White, true),
        //         'q' => self.castle.set_queenside(Team::Black, true),
        //         'k' => self.castle.set_kingside(Team::Black, true),
        //         '-' => {
        //             self.castle = CastleAvailability::none();
        //             break;
        //         }
        //         _ => panic!("Invalid")
        //     }
        // }


        match en_passant_string {
            "-" => self.en_passant = 0,
            _ =>
                match index_from_string(en_passant_string) {
                    Some(i) => self.en_passant |= 1 << i,
                    None => panic!("Invalid En Passant String"),
                }
        }

        let halfmoves = u16::from_str_radix(half_moves_string, 10);
        let fullmoves = u16::from_str_radix(full_moves_string, 10);

        if halfmoves.is_err() || fullmoves.is_err() {
            return Err("Invalid half or full moves");
        }
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
            self.all &= negative_mask;
            self.white.set_piece(piece, index);
            self.black.set_piece(ChessPiece::None, index);
        } else {
            self.all &= negative_mask;
            self.black.set_piece(piece, index);
            self.white.set_piece(ChessPiece::None, index);
        }
    }


    pub fn piece_array(&self) -> [ChessPiece; 64] {
        let mut arr = [ChessPiece::None; 64];
        
        for i in 0..64 {
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