use std::fmt::Display;

use crossterm::style::Stylize;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Team {
    White,
    Black
}

impl Team {
    pub fn other(&self) -> Team {
        match self {
            Team::White => Team::Black,
            Team::Black => Team::White,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChessPiece {
    None,
    Pawn(Team),
    Knight(Team),
    Bishop(Team),
    Rook(Team),
    Queen(Team),
    King(Team)
}

impl ChessPiece {
    pub fn is_pawn(&self) -> bool {
        if let ChessPiece::Pawn(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_knight(&self) -> bool {
        if let ChessPiece::Knight(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_bishop(&self) -> bool {
        if let ChessPiece::Bishop(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_rook(&self) -> bool {
        if let ChessPiece::Rook(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_queen(&self) -> bool {
        if let ChessPiece::Queen(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_king(&self) -> bool {
        if let ChessPiece::King(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_none(&self) -> bool {
        if let ChessPiece::None = self {
            true
        } else {
            false
        }
    }
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn team(&self) -> Team {
        match self {
            ChessPiece::None => panic!("Cannot retrieve team from none."),
            ChessPiece::Pawn(team) => *team,
            ChessPiece::Knight(team) => *team,
            ChessPiece::Bishop(team) => *team,
            ChessPiece::Rook(team) => *team,
            ChessPiece::Queen(team) => *team,
            ChessPiece::King(team) => *team,
        }
    }
    pub fn is_team(&self, team: Team) -> bool {
        match self {
            ChessPiece::None => false,
            ChessPiece::Pawn(_team) => *_team == team,
            ChessPiece::Knight(_team) => *_team == team,
            ChessPiece::Bishop(_team) => *_team == team,
            ChessPiece::Rook(_team) => *_team == team,
            ChessPiece::Queen(_team) => *_team == team,
            ChessPiece::King(_team) => *_team == team,
        }
    }

    pub fn fancy_uniform_char(&self) -> char {
        match self {
            ChessPiece::None => ' ',
            ChessPiece::Pawn(_) => '♙',
            ChessPiece::Knight(_) => '♘',
            ChessPiece::Bishop(_) => '♗',
            ChessPiece::Rook(_) => '♖',
            ChessPiece::Queen(_) => '♕',
            ChessPiece::King(_) => '♔',
        }
    }

    pub fn from_char(char: char) -> ChessPiece {
        match char {
            'p' => ChessPiece::Pawn(Team::Black),
            'n' => ChessPiece::Knight(Team::Black),
            'b' => ChessPiece::Bishop(Team::Black),
            'r' => ChessPiece::Rook(Team::Black),
            'q' => ChessPiece::Queen(Team::Black),
            'k' => ChessPiece::King(Team::Black),
            'P' => ChessPiece::Pawn(Team::White),
            'N' => ChessPiece::Knight(Team::White),
            'B' => ChessPiece::Bishop(Team::White),
            'R' => ChessPiece::Rook(Team::White),
            'Q' => ChessPiece::Queen(Team::White),
            'K' => ChessPiece::King(Team::White),
            _ => ChessPiece::None,
        }
    }
    pub fn char(&self) -> char {
        match self {
            ChessPiece::None => ' ',
            ChessPiece::Pawn(team) => match team {
                Team::White => 'P',
                Team::Black => 'p',
            },
            ChessPiece::Knight(team) => match team {
                Team::White => 'N',
                Team::Black => 'n',
            },
            ChessPiece::Bishop(team) => match team {
                Team::White => 'B',
                Team::Black => 'b',
            },
            ChessPiece::Rook(team) => match team {
                Team::White => 'R',
                Team::Black => 'r',
            },
            ChessPiece::Queen(team) => match team {
                Team::White => 'Q',
                Team::Black => 'q',
            },
            ChessPiece::King(team) => match team {
                Team::White => 'K',
                Team::Black => 'k',
            },
        }
    }
}

impl Display for ChessPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dis = if (self.is_none()) {
            ' '.stylize()
        } else {
            match self.team() {
                Team::White => self.fancy_uniform_char().yellow(),
                Team::Black => self.fancy_uniform_char().blue(),
            }
        };
        write!(f, "{}", dis)
    }
}