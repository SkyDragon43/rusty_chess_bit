use crate::{ChessBoard, castle::{CastleRights, CastleType}, chess::constants, piece::ChessPiece};




pub enum MoveType {
    Move,
    Pawn(Option<u8>),
    EnPassant(u8),
    Promotion(ChessPiece),
    Castle(CastleType),
}
pub struct ChessMove {
    pub from: u8,
    pub to: u8,
    pub move_type: MoveType,
}

pub struct PlayedMove {
    pub original: ChessMove,
    pub piece: ChessPiece,
    pub captured: ChessPiece,
    pub previous_en_passant: u64,
    pub previous_castle_rights: CastleRights,
}

impl PlayedMove {
    pub fn new(chess_move: ChessMove, piece: ChessPiece, captured: ChessPiece, previous_castle_rights: CastleRights, previous_en_passant: u64) -> PlayedMove {
        PlayedMove { original: chess_move, piece, captured, previous_en_passant, previous_castle_rights }
    }
}

impl ChessMove {
    pub fn new(from: u8, to: u8, move_type: MoveType) -> ChessMove {
        ChessMove {
            from,
            to,
            move_type,
        }
    }

    pub fn name(&self) -> String {
        if let MoveType::Castle(castle) = self.move_type {
            if castle.is_kingside() {
                return String::from("O-O");
            } else {
                return String::from("O-O-O");
            }
        }
        let mut name = String::new();

        let file = self.from % 8;
        let rank = self.from / 8;
        name.push(constants::file_char(file as i8));
        name.push(constants::rank_char(rank as i8));
        let file = self.to % 8;
        let rank = self.to / 8;
        name.push(constants::file_char(file as i8));
        name.push(constants::rank_char(rank as i8));
        if let MoveType::Promotion(promotion) = self.move_type {
            name.push(promotion.char());
        }

        name
    }
}