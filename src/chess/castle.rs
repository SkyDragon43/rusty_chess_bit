use crate::piece::Team;


pub const WHITE_KINGSIDE: u8 = 0b0001;
pub const WHITE_QUEENSIDE: u8 = 0b0010;
pub const BLACK_KINGSIDE: u8 = 0b0100;
pub const BLACK_QUEENSIDE: u8 = 0b1000;

#[derive(Clone, Copy, Debug)]
pub enum CastleType {
    WhiteKingSide,
    WhiteQueenSide,
    BlackKingSide,
    BlackQueenSide,
}
impl CastleType {
    pub fn team(&self) -> Team {
        match self {
            CastleType::WhiteKingSide => Team::White,
            CastleType::WhiteQueenSide => Team::White,
            CastleType::BlackKingSide => Team::Black,
            CastleType::BlackQueenSide => Team::Black,
        }
    }

    pub fn is_kingside(&self) -> bool {
        match self {
            CastleType::WhiteKingSide => true,
            CastleType::WhiteQueenSide => false,
            CastleType::BlackKingSide => true,
            CastleType::BlackQueenSide => false,
        }
    }

    pub fn queenside_of(team: Team) -> CastleType {
        match team {
            Team::Black => CastleType::BlackQueenSide,
            Team::White => CastleType::WhiteQueenSide
        }
    }
    pub fn kingside_of(team: Team) -> CastleType {
        match team {
            Team::Black => CastleType::BlackKingSide,
            Team::White => CastleType::WhiteKingSide
        }
    }


    pub fn get_new_king_index(&self) -> u8 {
        match self {
            CastleType::WhiteKingSide => 6,
            CastleType::WhiteQueenSide => 2,
            CastleType::BlackKingSide => 62,
            CastleType::BlackQueenSide => 58,
        }
    }
    pub fn get_new_rook_index(&self) -> u8 {
        match self {
            CastleType::WhiteKingSide => 5,
            CastleType::WhiteQueenSide => 3,
            CastleType::BlackKingSide => 61,
            CastleType::BlackQueenSide => 59,
        }
    }
    pub fn get_king_index(&self) -> u8 {
        match self {
            CastleType::WhiteKingSide => 4,
            CastleType::WhiteQueenSide => 4,
            CastleType::BlackKingSide => 60,
            CastleType::BlackQueenSide => 60,
        }
    }
    pub fn get_rook_index(&self) -> u8 {
        match self {
            CastleType::WhiteKingSide => 7,
            CastleType::WhiteQueenSide => 0,
            CastleType::BlackKingSide => 63,
            CastleType::BlackQueenSide => 56,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CastleRights {
    flags: u8,
}
impl CastleRights {
    pub fn get_queenside(&self, team: Team) -> bool {
        match team {
            Team::Black => self.flags & 0b1000 != 0,
            Team::White => self.flags & 0b0010 != 0,
        }
    }
    pub fn get_kingside(&self, team: Team) -> bool {
        match team {
            Team::Black => self.flags & 0b0100 != 0,
            Team::White => self.flags & 0b0001 != 0,
        }
    }
    pub fn set_queenside(&mut self, team: Team, value: bool) {
        match team {
            Team::Black => if value {
                self.flags |= 0b1000;
            } else {
                self.flags &= 0b0111;
            },
            Team::White => if value {
                self.flags |= 0b0010;
            } else {
                self.flags &= 0b1101;
            },
        };
    }
    pub fn set_kingside(&mut self, team: Team, value: bool) {
        match team {
            Team::Black => if value {
                self.flags |= 0b0100;
            } else {
                self.flags &= 0b1011;
            },
            Team::White => if value {
                self.flags |= 0b0001;
            } else {
                self.flags &= 0b1110;
            },
        };
    }
    pub fn set_team(&mut self, team: Team, value: bool) {
        match team {
            Team::Black => if value {
                self.flags |= 0b1100;
            } else {
                self.flags &= 0b0011;
            },
            Team::White => if value {
                self.flags |= 0b0011;
            } else {
                self.flags &= 0b1100;
            },
        };
    }
    pub fn set(&mut self, castle: CastleType, value: bool) {
        match castle {
            CastleType::WhiteKingSide => self.set_kingside(Team::White, value),
            CastleType::WhiteQueenSide => self.set_queenside(Team::White, value),
            CastleType::BlackKingSide => self.set_kingside(Team::Black, value),
            CastleType::BlackQueenSide => self.set_queenside(Team::Black, value),
        }
    }
    pub fn get(&self, castle: CastleType) -> bool {
        match castle {
            CastleType::WhiteKingSide => self.get_kingside(Team::White),
            CastleType::WhiteQueenSide => self.get_queenside(Team::White),
            CastleType::BlackKingSide => self.get_kingside(Team::Black),
            CastleType::BlackQueenSide => self.get_queenside(Team::Black),
        }
    }
    
    pub fn new(white_king_side: bool, white_queen_side: bool, black_king_side: bool, black_queen_side: bool) -> CastleRights {
        let flags:u8 = 
            if white_king_side  { 0b0001 } else { 0b0000 } |
            if white_queen_side { 0b0010 } else { 0b0000 } |
            if black_king_side  { 0b0100 } else { 0b0000 } |
            if black_queen_side { 0b1000 } else { 0b0000 };
        CastleRights { flags }
    }
    pub fn none() -> CastleRights {
        CastleRights { flags: 0 }
    }
    pub fn initial() -> CastleRights {
        CastleRights::new(true, true, true, true)
    }
    pub fn as_string(&self) -> String {
        if self.flags == 0 {
            return String::from("-");
        }
        let mut str = String::new();
        if self.get_queenside(Team::White) {
            str = str + "Q";
        }
        if self.get_kingside(Team::White) {
            str = str + "K";
        }
        if self.get_queenside(Team::Black) {
            str = str + "q";
        }
        if self.get_kingside(Team::Black) {
            str = str + "k";
        }

        str
    }
}