pub const FILE_A: u64 = 0x0101010101010101 << 0;
pub const FILE_B: u64 = 0x0101010101010101 << 1;
pub const FILE_C: u64 = 0x0101010101010101 << 2;
pub const FILE_D: u64 = 0x0101010101010101 << 3;
pub const FILE_E: u64 = 0x0101010101010101 << 4;
pub const FILE_F: u64 = 0x0101010101010101 << 5;
pub const FILE_G: u64 = 0x0101010101010101 << 6;
pub const FILE_H: u64 = 0x0101010101010101 << 7;
pub const FILES: [u64; 8] = [FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H];
pub const RANK_1: u64 = 0x00000000000000FF << 0;
pub const RANK_2: u64 = 0x00000000000000FF << 8;
pub const RANK_3: u64 = 0x00000000000000FF << 16;
pub const RANK_4: u64 = 0x00000000000000FF << 24;
pub const RANK_5: u64 = 0x00000000000000FF << 32;
pub const RANK_6: u64 = 0x00000000000000FF << 40;
pub const RANK_7: u64 = 0x00000000000000FF << 48;
pub const RANK_8: u64 = 0x00000000000000FF << 56;
pub const RANKS: [u64; 8] = [RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8];

/**
 *  0 0 0 1
 *  0 0 1 0
 *  0 1 0 0
 *  1 0 0 0
 * Top left is zero
 */
pub const DIAGONAL_0: u64 = 0x0000000000000001;
pub const DIAGONAL_1: u64 = DIAGONAL_0 << 1 | DIAGONAL_0 << 8;
pub const DIAGONAL_2: u64 = DIAGONAL_1 << 1 | DIAGONAL_1 << 8;
pub const DIAGONAL_3: u64 = DIAGONAL_2 << 1 | DIAGONAL_2 << 8;
pub const DIAGONAL_4: u64 = DIAGONAL_3 << 1 | DIAGONAL_3 << 8;
pub const DIAGONAL_5: u64 = DIAGONAL_4 << 1 | DIAGONAL_4 << 8;
pub const DIAGONAL_6: u64 = DIAGONAL_5 << 1 | DIAGONAL_5 << 8;
pub const DIAGONAL_7: u64 = DIAGONAL_6 << 1 | DIAGONAL_6 << 8;
pub const DIAGONAL_8: u64 = DIAGONAL_7 << 8;
pub const DIAGONAL_9: u64 = DIAGONAL_8 << 8;
pub const DIAGONAL_10: u64 = DIAGONAL_9 << 8;
pub const DIAGONAL_11: u64 = DIAGONAL_10 << 8;
pub const DIAGONAL_12: u64 = DIAGONAL_11 << 8;
pub const DIAGONAL_13: u64 = DIAGONAL_12 << 8;
pub const DIAGONAL_14: u64 = DIAGONAL_13 << 8;
pub const DIAGONALS: [u64; 15] = 
    [DIAGONAL_0, DIAGONAL_1, DIAGONAL_2, DIAGONAL_3, DIAGONAL_4, DIAGONAL_5, DIAGONAL_6, DIAGONAL_7, 
     DIAGONAL_8, DIAGONAL_9, DIAGONAL_10, DIAGONAL_11, DIAGONAL_12, DIAGONAL_13, DIAGONAL_14];

/**
 *  1 0 0 0
 *  0 1 0 0
 *  0 0 1 0
 *  0 0 0 1
 * Top left is zero
 */
pub const ANTI_DIAGONAL_0: u64 = 0x0000000000000080;
pub const ANTI_DIAGONAL_1: u64 = ANTI_DIAGONAL_0 << 8 | ANTI_DIAGONAL_0 >> 1;
pub const ANTI_DIAGONAL_2: u64 = ANTI_DIAGONAL_1 << 8 | ANTI_DIAGONAL_1 >> 1;
pub const ANTI_DIAGONAL_3: u64 = ANTI_DIAGONAL_2 << 8 | ANTI_DIAGONAL_2 >> 1;
pub const ANTI_DIAGONAL_4: u64 = ANTI_DIAGONAL_3 << 8 | ANTI_DIAGONAL_3 >> 1;
pub const ANTI_DIAGONAL_5: u64 = ANTI_DIAGONAL_4 << 8 | ANTI_DIAGONAL_4 >> 1;
pub const ANTI_DIAGONAL_6: u64 = ANTI_DIAGONAL_5 << 8 | ANTI_DIAGONAL_5 >> 1;
pub const ANTI_DIAGONAL_7: u64 = ANTI_DIAGONAL_6 << 8 | ANTI_DIAGONAL_6 >> 1;
pub const ANTI_DIAGONAL_8: u64 = ANTI_DIAGONAL_7 << 8;
pub const ANTI_DIAGONAL_9: u64 = ANTI_DIAGONAL_8 << 8;
pub const ANTI_DIAGONAL_10: u64 = ANTI_DIAGONAL_9 << 8;
pub const ANTI_DIAGONAL_11: u64 = ANTI_DIAGONAL_10 << 8;
pub const ANTI_DIAGONAL_12: u64 = ANTI_DIAGONAL_11 << 8;
pub const ANTI_DIAGONAL_13: u64 = ANTI_DIAGONAL_12 << 8;
pub const ANTI_DIAGONAL_14: u64 = ANTI_DIAGONAL_13 << 8;
pub const ANTI_DIAGONALS: [u64; 15] = 
    [ANTI_DIAGONAL_0, ANTI_DIAGONAL_1, ANTI_DIAGONAL_2, ANTI_DIAGONAL_3, ANTI_DIAGONAL_4, ANTI_DIAGONAL_5, ANTI_DIAGONAL_6, ANTI_DIAGONAL_7, 
     ANTI_DIAGONAL_8, ANTI_DIAGONAL_9, ANTI_DIAGONAL_10, ANTI_DIAGONAL_11, ANTI_DIAGONAL_12, ANTI_DIAGONAL_13, ANTI_DIAGONAL_14];


pub fn char_file(file: char) -> i8 {
    match file {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => -1,
        _ => -1,
    }
}
pub fn char_rank(rank: char) -> i8 {
    match rank {
        '1' => 0,
        '2' => 1,
        '3' => 2,
        '4' => 3,
        '5' => 4,
        '6' => 5,
        '7' => 6,
        '8' => -1,
        _ => -1,
    }
}
pub fn file_char(x: i8) -> char {
    match x {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => '\0',
    }
}
pub fn rank_char(y: i8) -> char {
    match y {
        0 => '1',
        1 => '2',
        2 => '3',
        3 => '4',
        4 => '5',
        5 => '6',
        6 => '7',
        7 => '8',
        _ => '\0',
    }
}
pub fn index_from_string(coord: &str) -> Option<u8> {
    let mut chars = coord.chars();
    let file = chars.next()?;
    let rank = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    let x = char_file(file);
    let y = char_rank(rank);
    if x < 0 || y < 0 {
        return None;
    }
    return Some((x as u8) + (y as u8) * 8);
}