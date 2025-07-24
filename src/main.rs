use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event as CrosstermEvent, KeyCode, MouseEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

#[derive(Clone)]
struct Board {
    squares: [[Option<Piece>; 8]; 8],
    captured_white: Vec<Piece>,
    captured_black: Vec<Piece>,
    current_turn: ColorChess,
    white_points: u32,
    black_points: u32,
    // fields for castling and en passant
    white_king_moved: bool,
    black_king_moved: bool,
    white_rook_king_side_moved: bool,
    white_rook_queen_side_moved: bool,
    black_rook_king_side_moved: bool,
    black_rook_queen_side_moved: bool,
    en_passant_target: Option<(usize, usize)>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ColorChess {
    White,
    Black,
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Piece(u8);

// Piece type constants (bits 0-2)
const PAWN: u8 = 0b000;
const KNIGHT: u8 = 0b001;
const BISHOP: u8 = 0b010;
const ROOK: u8 = 0b011;
const QUEEN: u8 = 0b100;
const KING: u8 = 0b101;

// Color flag (bit 3)
const WHITE_FLAG: u8 = 0b0000;
const BLACK_FLAG: u8 = 0b1000;

impl Piece {
    // Constructor
    pub fn new(piece_type: PieceType, color: ColorChess) -> Self {
        let type_bits = match piece_type {
            PieceType::Pawn => PAWN,
            PieceType::Knight => KNIGHT,
            PieceType::Bishop => BISHOP,
            PieceType::Rook => ROOK,
            PieceType::Queen => QUEEN,
            PieceType::King => KING,
        };

        let color_bit = match color {
            ColorChess::White => WHITE_FLAG,
            ColorChess::Black => BLACK_FLAG,
        };

        Piece(type_bits | color_bit)
    }

    // Getters
    pub fn piece_type(&self) -> PieceType {
        match self.0 & 0b0111 {
            PAWN => PieceType::Pawn,
            KNIGHT => PieceType::Knight,
            BISHOP => PieceType::Bishop,
            ROOK => PieceType::Rook,
            QUEEN => PieceType::Queen,
            KING => PieceType::King,
            _ => unreachable!("Invalid piece type bits"),
        }
    }

    pub fn color(&self) -> ColorChess {
        if (self.0 & BLACK_FLAG) != 0 {
            ColorChess::Black
        } else {
            ColorChess::White
        }
    }

    pub fn is_color(&self, color: ColorChess) -> bool {
        self.color() == color
    }

    pub fn is_type(&self, piece_type: PieceType) -> bool {
        self.piece_type() == piece_type
    }

    fn to_char(&self) -> char {
        match self.piece_type() {
            PieceType::King => '♚',
            PieceType::Queen => '♛',
            PieceType::Rook => '♜',
            PieceType::Bishop => '♝',
            PieceType::Knight => '♞',
            PieceType::Pawn => '♟',
        }
    }

    fn points(&self) -> u32 {
        match self.piece_type() {
            PieceType::Pawn => 1,
            PieceType::Knight | PieceType::Bishop => 3,
            PieceType::Rook => 5,
            PieceType::Queen => 9,
            PieceType::King => 0, // King's value is infinite in terms of game points
        }
    }
}

impl Board {
    fn new() -> Board {
        let mut squares = [[None; 8]; 8];
        for i in 0..8 {
            squares[1][i] = Some(Piece::new(PieceType::Pawn, ColorChess::White));
            squares[6][i] = Some(Piece::new(PieceType::Pawn, ColorChess::Black));
        }

        let back_rank = [
            PieceType::Rook,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Queen,
            PieceType::King,
            PieceType::Bishop,
            PieceType::Knight,
            PieceType::Rook,
        ];

        for (i, &piece_type) in back_rank.iter().enumerate() {
            squares[0][i] = Some(Piece::new(piece_type, ColorChess::White));
            squares[7][i] = Some(Piece::new(piece_type, ColorChess::Black));
        }

        Board {
            squares,
            captured_white: Vec::new(),
            captured_black: Vec::new(),
            current_turn: ColorChess::White,
            white_points: 0,
            black_points: 0,
            white_king_moved: false,
            black_king_moved: false,
            white_rook_king_side_moved: false,
            white_rook_queen_side_moved: false,
            black_rook_king_side_moved: false,
            black_rook_queen_side_moved: false,
            en_passant_target: None,
        }
    }

    fn choose_player_color() -> ColorChess {
        ColorChess::White
    }

    fn is_valid_move(&self, start: (usize, usize), end: (usize, usize), color: ColorChess) -> bool {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        if start == end || end_x >= 8 || end_y >= 8 {
            return false;
        }
        if let Some(piece) = &self.squares[start_x][start_y] {
            if piece.color() != color {
                return false;
            }
            match piece.piece_type() {
                PieceType::Pawn => self.is_valid_pawn_move(start, end, color),
                PieceType::Knight => self.is_valid_knight_move(start, end, color),
                PieceType::Bishop => self.is_valid_bishop_move(start, end, color),
                PieceType::Rook => self.is_valid_rook_move(start, end, color),
                PieceType::Queen => self.is_valid_queen_move(start, end, color),
                PieceType::King => self.is_valid_king_move(start, end, color),
            }
        } else {
            false
        }
    }

    fn move_piece(&mut self, start: (usize, usize), end: (usize, usize)) {
        self.en_passant_target = None;
        let piece_moving_clone = self.squares[start.0][start.1].clone();

        // Track king and rook movements for castling validity
        if let Some(piece_moving) = piece_moving_clone {
            if piece_moving.is_type(PieceType::King) {
                if piece_moving.color() == ColorChess::White {
                    self.white_king_moved = true;
                } else {
                    self.black_king_moved = true;
                }
                if (start.1 as isize - end.1 as isize).abs() == 2 {
                    // King-side castling
                    if end.1 == 6 {
                        let rook = self.squares[start.0][7].take();
                        self.squares[start.0][5] = rook;
                    }
                    // Queen-side castling
                    else if end.1 == 2 {
                        let rook = self.squares[start.0][0].take();
                        self.squares[start.0][3] = rook;
                    }
                }
            } else if piece_moving.is_type(PieceType::Rook) {
                if piece_moving.color() == ColorChess::White {
                    if start == (0, 0) {
                        self.white_rook_queen_side_moved = true;
                    } else if start == (0, 7) {
                        self.white_rook_king_side_moved = true;
                    }
                } else {
                    // Black rook
                    if start == (7, 0) {
                        self.black_rook_queen_side_moved = true;
                    } else if start == (7, 7) {
                        self.black_rook_king_side_moved = true;
                    }
                }
            }
            // Set en_passant_target if a pawn moves two squares
            if piece_moving.is_type(PieceType::Pawn) {
                if piece_moving.color() == ColorChess::White && start.0 == 1 && end.0 == 3 {
                    self.en_passant_target = Some((2, start.1)); // Square behind white pawn
                } else if piece_moving.color() == ColorChess::Black && start.0 == 6 && end.0 == 4 {
                    self.en_passant_target = Some((5, start.1)); // Square behind black pawn
                }
            }
        }

        // Handle en passant capture
        if let Some(piece_moving) = self.squares[start.0][start.1] {
            if piece_moving.is_type(PieceType::Pawn) {
                if (start.1 as isize - end.1 as isize).abs() == 1
                    && self.squares[end.0][end.1].is_none()
                {
                    // This is a diagonal move to an empty square, must be en passant
                    let captured_pawn_pos = if piece_moving.color() == ColorChess::White {
                        (end.0 - 1, end.1) // Pawn was at start_x (row 4) and moved to end_x (row 5)
                    } else {
                        (end.0 + 1, end.1) // Pawn was at start_x (row 3) and moved to end_x (row 2)
                    };

                    if let Some(captured) =
                        self.squares[captured_pawn_pos.0][captured_pawn_pos.1].take()
                    {
                        if captured.color() == ColorChess::White {
                            self.captured_white.push(captured);
                            self.white_points += captured.points();
                        } else {
                            self.captured_black.push(captured);
                            self.black_points += captured.points();
                        }
                    }
                }
            }
        }

        // Capture logic for regular moves
        if let Some(captured) = self.squares[end.0][end.1].take() {
            if captured.color() == ColorChess::White {
                self.captured_white.push(captured);
                self.white_points += captured.points();
            } else {
                self.captured_black.push(captured);
                self.black_points += captured.points();
            }
        }

        // Move the piece
        if let Some(piece) = self.squares[start.0][start.1].take() {
            self.squares[end.0][end.1] = Some(piece);
        }

        // Pawn promotion
        if let Some(piece) = &self.squares[end.0][end.1] {
            if piece.is_type(PieceType::Pawn) {
                if (piece.color() == ColorChess::White && end.0 == 7)
                    || (piece.color() == ColorChess::Black && end.0 == 0)
                {
                    // For simplicity, auto-promote to Queen. In a full game, you'd prompt the user.
                    self.squares[end.0][end.1] = Some(Piece::new(PieceType::Queen, piece.color()));
                }
            }
        }
    }

    fn get_all_moves(&self, color: ColorChess) -> Vec<((usize, usize), (usize, usize))> {
        let mut moves = Vec::new();
        for start_x in 0..8 {
            for start_y in 0..8 {
                if let Some(piece) = &self.squares[start_x][start_y] {
                    if piece.color() == color {
                        for end_x in 0..8 {
                            for end_y in 0..8 {
                                if self.is_valid_move((start_x, start_y), (end_x, end_y), color) {
                                    moves.push(((start_x, start_y), (end_x, end_y)));
                                }
                            }
                        }
                    }
                }
            }
        }
        moves
    }

    fn is_valid_pawn_move(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        color: ColorChess,
    ) -> bool {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        // Standard pawn moves
        if color == ColorChess::White {
            // One step forward
            if start_x + 1 == end_x && start_y == end_y && self.squares[end_x][end_y].is_none() {
                return true;
            }
            // Two steps forward from starting position
            if start_x == 1
                && end_x == 3
                && start_y == end_y
                && self.squares[2][end_y].is_none()
                && self.squares[end_x][end_y].is_none()
            {
                return true;
            }
            // Capturing diagonally
            if start_x + 1 == end_x && (start_y as isize - end_y as isize).abs() == 1 {
                if let Some(piece) = &self.squares[end_x][end_y] {
                    if piece.color() == ColorChess::Black {
                        return true;
                    }
                }
            }
        } else {
            // Black pawn
            // One step forward
            if start_x > 0
                && start_x - 1 == end_x
                && start_y == end_y
                && self.squares[end_x][end_y].is_none()
            {
                return true;
            }
            // Two steps forward from starting position
            if start_x == 6
                && end_x == 4
                && start_y == end_y
                && self.squares[5][end_y].is_none()
                && self.squares[end_x][end_y].is_none()
            {
                return true;
            }
            // Capturing diagonally
            if start_x > 0 && start_x - 1 == end_x && (start_y as isize - end_y as isize).abs() == 1
            {
                if let Some(piece) = &self.squares[end_x][end_y] {
                    if piece.color() == ColorChess::White {
                        return true;
                    }
                }
            }
        }

        // En passant
        if (start_y as isize - end_y as isize).abs() == 1 {
            if let Some(target) = self.en_passant_target {
                if color == ColorChess::White {
                    if start_x == 4 && end_x == 5 && end == target {
                        // Check if the pawn to be captured is actually there
                        if let Some(pawn_to_capture) = &self.squares[start_x][end_y] {
                            if pawn_to_capture.is_type(PieceType::Pawn)
                                && pawn_to_capture.is_color(ColorChess::Black)
                            {
                                return true;
                            }
                        }
                    }
                } else {
                    // Black pawn
                    if start_x == 3 && end_x == 2 && end == target {
                        // Check if the pawn to be captured is actually there
                        if let Some(pawn_to_capture) = &self.squares[start_x][end_y] {
                            if pawn_to_capture.is_type(PieceType::Pawn)
                                && pawn_to_capture.is_color(ColorChess::White)
                            {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    fn is_valid_bishop_move(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        color: ColorChess,
    ) -> bool {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        if (start_x as isize - end_x as isize).abs() != (start_y as isize - end_y as isize).abs() {
            return false;
        }

        let dx = if end_x > start_x { 1 } else { -1 };
        let dy = if end_y > start_y { 1 } else { -1 };

        let mut x = start_x as isize + dx;
        let mut y = start_y as isize + dy;

        while (x != end_x as isize) && (y != end_y as isize) {
            if self.squares[x as usize][y as usize].is_some() {
                return false;
            }
            x += dx;
            y += dy;
        }

        self.squares[end_x][end_y].is_none()
            || self.squares[end_x][end_y].map_or(false, |p| p.color() != color)
    }

    fn is_valid_rook_move(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        color: ColorChess,
    ) -> bool {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        if start_x != end_x && start_y != end_y {
            return false;
        }

        if start_x == end_x {
            let range = if start_y < end_y {
                start_y + 1..end_y
            } else {
                end_y + 1..start_y
            };
            for y in range {
                if self.squares[start_x][y].is_some() {
                    return false;
                }
            }
        } else {
            let range = if start_x < end_x {
                start_x + 1..end_x
            } else {
                end_x + 1..start_x
            };
            for x in range {
                if self.squares[x][start_y].is_some() {
                    return false;
                }
            }
        }

        if let Some(piece) = &self.squares[end_x][end_y] {
            return piece.color() != color;
        }

        true
    }

    fn is_valid_knight_move(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        color: ColorChess,
    ) -> bool {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        let dx = (end_x as isize - start_x as isize).abs();
        let dy = (end_y as isize - start_y as isize).abs();

        if (dx == 2 && dy == 1) || (dx == 1 && dy == 2) {
            return self.squares[end_x][end_y].is_none()
                || self.squares[end_x][end_y].map_or(false, |p| p.color() != color);
        }
        false
    }

    fn is_valid_queen_move(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        color: ColorChess,
    ) -> bool {
        self.is_valid_rook_move(start, end, color) || self.is_valid_bishop_move(start, end, color)
    }

    fn is_valid_king_move(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        color: ColorChess,
    ) -> bool {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        // Check for castling first
        if self.is_valid_castling(start, end, color) {
            return true;
        }

        let dx = (end_x as isize - start_x as isize).abs();
        let dy = (end_y as isize - start_y as isize).abs();

        if dx <= 1 && dy <= 1 {
            if let Some(piece) = &self.squares[end_x][end_y] {
                piece.color() != color
            } else {
                true
            }
        } else {
            false
        }
    }

    fn is_square_attacked(
        &self,
        target_square: (usize, usize),
        attacker_color: ColorChess,
    ) -> bool {
        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = &self.squares[x][y] {
                    if piece.color() == attacker_color {
                        let mut temp_board_for_attack_check = self.clone();
                        let temp_target_piece = temp_board_for_attack_check.squares
                            [target_square.0][target_square.1]
                            .take();

                        let is_attacked = temp_board_for_attack_check.is_valid_move(
                            (x, y),
                            target_square,
                            attacker_color,
                        );

                        temp_board_for_attack_check.squares[target_square.0][target_square.1] =
                            temp_target_piece;

                        if is_attacked {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn find_king(&self, color: ColorChess) -> Option<(usize, usize)> {
        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = &self.squares[x][y] {
                    if piece.is_type(PieceType::King) && piece.is_color(color) {
                        return Some((x, y));
                    }
                }
            }
        }
        None
    }

    fn is_in_check(&self, color: ColorChess) -> bool {
        let king_position = match self.find_king(color) {
            Some(pos) => pos,
            None => return false,
        };

        let opponent_color = if color == ColorChess::White {
            ColorChess::Black
        } else {
            ColorChess::White
        };

        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = &self.squares[x][y] {
                    if piece.color() == opponent_color {
                        if self.is_valid_move((x, y), king_position, opponent_color) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn is_checkmate(&mut self, color: ColorChess) -> bool {
        if self.find_king(color).is_none() {
            return true;
        }

        if !self.is_in_check(color) {
            return false;
        }

        self.get_all_legal_moves(color).is_empty()
    }

    fn make_move_for_test(&mut self, start: (usize, usize), end: (usize, usize)) {
        // Simulate en passant capture if it's an en passant move
        if let Some(piece_moving) = self.squares[start.0][start.1] {
            if piece_moving.is_type(PieceType::Pawn) {
                if (start.1 as isize - end.1 as isize).abs() == 1
                    && self.squares[end.0][end.1].is_none()
                {
                    // This is a diagonal move to an empty square, must be en passant
                    let captured_pawn_pos = if piece_moving.color() == ColorChess::White {
                        (end.0 - 1, end.1)
                    } else {
                        (end.0 + 1, end.1)
                    };
                    self.squares[captured_pawn_pos.0][captured_pawn_pos.1] = None;
                }
            }
        }

        // Move the piece
        let piece = self.squares[start.0][start.1].take();
        self.squares[end.0][end.1] = piece;

        // Simulate castling rook move
        if let Some(moved_piece) = piece {
            if moved_piece.is_type(PieceType::King) {
                if (start.1 as isize - end.1 as isize).abs() == 2 {
                    // King-side castling
                    if end.1 == 6 {
                        let rook = self.squares[start.0][7].take();
                        self.squares[start.0][5] = rook;
                    }
                    // Queen-side castling
                    else if end.1 == 2 {
                        let rook = self.squares[start.0][0].take();
                        self.squares[start.0][3] = rook;
                    }
                }
            }
        }
    }

    fn is_stalemate(&self, color: ColorChess) -> bool {
        if self.is_in_check(color) {
            return false;
        }
        self.get_all_legal_moves(color).is_empty()
    }

    fn has_king(&self, color: ColorChess) -> bool {
        self.find_king(color).is_some()
    }

    fn get_all_legal_moves(&self, color: ColorChess) -> Vec<((usize, usize), (usize, usize))> {
        let mut legal_moves = Vec::new();
        for start_x in 0..8 {
            for start_y in 0..8 {
                if let Some(piece) = &self.squares[start_x][start_y] {
                    if piece.color() == color {
                        for end_x in 0..8 {
                            for end_y in 0..8 {
                                if self.is_valid_move((start_x, start_y), (end_x, end_y), color) {
                                    let mut temp_board = self.clone();
                                    temp_board
                                        .make_move_for_test((start_x, start_y), (end_x, end_y));

                                    if !temp_board.is_in_check(color) {
                                        legal_moves.push(((start_x, start_y), (end_x, end_y)));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        legal_moves
    }

    fn is_game_over(&mut self, color: ColorChess) -> bool {
        if self.is_checkmate(color) {
            return true;
        }
        if self.is_stalemate(color) {
            return true;
        }
        // TODO: Add other game-ending conditions here if necessary (e.g., insufficient material)
        false
    }

    // This method is for text input, will be less used with mouse input
    fn parse_move(&self, move_str: &str) -> Option<(usize, usize)> {
        if move_str.len() != 2 {
            return None;
        }

        let chars: Vec<char> = move_str.chars().collect();
        let col = chars[0].to_ascii_lowercase();
        let row = chars[1];

        if !('a'..='h').contains(&col) || !('1'..='8').contains(&row) {
            return None;
        }

        let col_index = (col as usize) - ('a' as usize);
        let row_index = 8 - (row.to_digit(10)? as usize);

        Some((row_index, col_index))
    }

    fn switch_turn(&mut self) {
        self.current_turn = match self.current_turn {
            ColorChess::White => ColorChess::Black,
            ColorChess::Black => ColorChess::White,
        };
    }

    fn get_current_turn(&self) -> ColorChess {
        self.current_turn
    }

    fn is_valid_castling(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        color: ColorChess,
    ) -> bool {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        // King must be at its starting position
        let (king_start_x, king_start_y) = if color == ColorChess::White {
            (0, 4)
        } else {
            (7, 4)
        };
        if start != (king_start_x, king_start_y) {
            return false;
        }

        // King and selected rook must not have moved
        if color == ColorChess::White {
            if self.white_king_moved {
                return false;
            }
            if end == (0, 6) {
                // King-side castling (White)
                if self.white_rook_king_side_moved {
                    return false;
                }
                if self.squares[0][5].is_some() || self.squares[0][6].is_some() {
                    return false;
                }
                if self.is_in_check(color) ||
                   self.is_square_attacked((0, 5), ColorChess::Black) || // Square king passes through
                   self.is_square_attacked((0, 6), ColorChess::Black)
                {
                    // Square king lands on
                    return false;
                }
                return true;
            } else if end == (0, 2) {
                // Queen-side castling (White)
                if self.white_rook_queen_side_moved {
                    return false;
                }
                if self.squares[0][1].is_some()
                    || self.squares[0][2].is_some()
                    || self.squares[0][3].is_some()
                {
                    return false;
                }
                // Check if king passes through or lands on attacked square
                if self.is_in_check(color) ||
                   self.is_square_attacked((0, 3), ColorChess::Black) || // Square king passes through
                   self.is_square_attacked((0, 2), ColorChess::Black)
                {
                    // Square king lands on
                    return false;
                }
                return true;
            }
        } else {
            // Black
            if self.black_king_moved {
                return false;
            }
            if end == (7, 6) {
                // King-side castling (Black)
                if self.black_rook_king_side_moved {
                    return false;
                }
                if self.squares[7][5].is_some() || self.squares[7][6].is_some() {
                    return false;
                }
                // Check if king passes through or lands on attacked square
                if self.is_in_check(color)
                    || self.is_square_attacked((7, 5), ColorChess::White)
                    || self.is_square_attacked((7, 6), ColorChess::White)
                {
                    return false;
                }
                return true;
            } else if end == (7, 2) {
                // Queen-side castling (Black)
                if self.black_rook_queen_side_moved {
                    return false;
                }
                if self.squares[7][1].is_some()
                    || self.squares[7][2].is_some()
                    || self.squares[7][3].is_some()
                {
                    return false;
                }
                // Check if king passes through or lands on attacked square
                if self.is_in_check(color)
                    || self.is_square_attacked((7, 3), ColorChess::White)
                    || self.is_square_attacked((7, 2), ColorChess::White)
                {
                    return false;
                }
                return true;
            }
        }
        false
    }
}

// --- TUI Application State ---
struct App {
    board: Board,
    player_perspective: ColorChess,
    selected_square: Option<(usize, usize)>, // (row, col) of the currently selected piece
    message: String,
    game_over_message: Option<String>,
    // Store all legal moves for the currently selected piece for highlighting
    possible_moves: Vec<(usize, usize)>,
}

impl App {
    fn new() -> App {
        let board = Board::new();
        let player_perspective = Board::choose_player_color();
        App {
            board,
            player_perspective,
            selected_square: None,
            message: "Welcome to Chess! Click a piece to move.".to_string(),
            game_over_message: None,
            possible_moves: Vec::new(),
        }
    }

    fn handle_mouse_click(&mut self, mouse_x: u16, mouse_y: u16) {
        if self.game_over_message.is_some() {
            self.message = "Game is over! Press 'q' to quit.".to_string();
            return;
        }

        // Define constants for square dimensions (must match ui function)
        const SQUARE_WIDTH: u16 = 6;
        const SQUARE_HEIGHT: u16 = 4;

        // Get current terminal size to replicate the UI layout calculation
        let (term_width, term_height) = match crossterm::terminal::size() {
            Ok(size) => size,
            Err(_) => {
                self.message = "Could not get terminal size.".to_string();
                return;
            }
        };

        let frame_size = tui::layout::Rect::new(0, 0, term_width, term_height);

        // Replicate the layout calculation from the ui function
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(8), // Captured pieces and info
                    Constraint::Min(0),    // Chess board (takes remaining space)
                    Constraint::Length(3), // Messages and input
                ]
                .as_ref(),
            )
            .split(frame_size);

        let board_block = Block::default()
            .borders(Borders::ALL)
            .title(" Chess Board ");

        // Get the inner area of the board block, which is where the actual squares are drawn
        let board_area = board_block.inner(chunks[1]);

        const BOARD_INNER_VISUAL_OFFSET_COL: u16 = 3; // ' ' (padding) + 'a' (file label) + ' ' (spacing)
        const BOARD_INNER_VISUAL_OFFSET_ROW: u16 = 1; // '8' (rank label)

        // Calculate clicked coordinates relative to the *start of the actual board squares*
        let effective_board_start_x = board_area.x + BOARD_INNER_VISUAL_OFFSET_COL;
        let effective_board_start_y = board_area.y + BOARD_INNER_VISUAL_OFFSET_ROW;

        // Check if the click is within the calculated effective board area
        if mouse_y >= effective_board_start_y &&
           mouse_y < effective_board_start_y + (8 * SQUARE_HEIGHT) && // 8 ranks * SQUARE_HEIGHT
           mouse_x >= effective_board_start_x &&
           mouse_x < effective_board_start_x + (8 * SQUARE_WIDTH)
        {
            // 8 squares * SQUARE_WIDTH

            let clicked_relative_row = mouse_y - effective_board_start_y;
            let clicked_relative_col = mouse_x - effective_board_start_x;

            // Convert relative terminal coordinates to board coordinates (0-7)
            let board_row = 7 - (clicked_relative_row as usize / SQUARE_HEIGHT as usize); // Divide by SQUARE_HEIGHT
            let board_col = clicked_relative_col as usize / SQUARE_WIDTH as usize; // Divide by SQUARE_WIDTH

            self.handle_board_click((board_row, board_col));
        } else {
            self.message = format!("Clicked outside board: ({}, {}).", mouse_x, mouse_y);
        }
    }

    fn handle_board_click(&mut self, clicked_square: (usize, usize)) {
        if self.game_over_message.is_some() {
            self.message = "Game is over! Press 'q' to quit.".to_string();
            return;
        }

        let (r, c) = clicked_square;
        let current_turn_color = self.board.get_current_turn();

        if let Some(start_sq) = self.selected_square {
            // Second click: attempt to make a move
            let end_sq = clicked_square;

            let mut temp_board_for_legality_check = self.board.clone();
            temp_board_for_legality_check.make_move_for_test(start_sq, end_sq);

            if self
                .board
                .is_valid_move(start_sq, end_sq, current_turn_color)
                && !temp_board_for_legality_check.is_in_check(current_turn_color)
            {
                self.board.move_piece(start_sq, end_sq);
                self.message = format!(
                    "Player {:?} moved {}{}-{}{}",
                    current_turn_color,
                    (b'a' + start_sq.1 as u8) as char,
                    8 - start_sq.0,
                    (b'a' + end_sq.1 as u8) as char,
                    8 - end_sq.0
                );

                // After a valid move, check for checkmate/stalemate on the *opponent's* turn
                let opponent_color = match current_turn_color {
                    ColorChess::White => ColorChess::Black,
                    ColorChess::Black => ColorChess::White,
                };

                if self.board.is_checkmate(opponent_color) {
                    self.game_over_message =
                        Some(format!("Checkmate! {:?} wins.", current_turn_color));
                    self.message = self.game_over_message.clone().unwrap();
                } else if self.board.is_stalemate(opponent_color) {
                    self.game_over_message = Some("Stalemate! The game is a draw.".to_string());
                    self.message = self.game_over_message.clone().unwrap();
                }
                self.board.switch_turn();
                self.selected_square = None; // Reset selection
                self.possible_moves.clear(); // Clear highlights
            } else {
                self.message =
                    "Invalid move, or this move puts your king in check. Try again.".to_string();
                self.selected_square = None; // Clear selection on invalid second click
                self.possible_moves.clear(); // Clear highlights
            }
        } else {
            // First click: select a piece
            if let Some(piece) = &self.board.squares[r][c] {
                if piece.color() == current_turn_color {
                    self.selected_square = Some(clicked_square);
                    self.message = format!(
                        "Selected {:?} at {}{}. Now click destination.",
                        piece.piece_type(),
                        (b'a' + c as u8) as char,
                        8 - r
                    );
                    // Calculate and store legal moves for highlighting
                    self.possible_moves = self
                        .board
                        .get_all_legal_moves(current_turn_color)
                        .into_iter()
                        .filter(|(start, _)| *start == clicked_square)
                        .map(|(_, end)| end)
                        .collect();
                } else {
                    self.message = format!(
                        "That's not your piece. It's {:?}'s turn.",
                        current_turn_color
                    );
                    self.selected_square = None;
                    self.possible_moves.clear();
                }
            } else {
                self.message = "No piece at that square. Click a piece to move.".to_string();
                self.selected_square = None;
                self.possible_moves.clear();
            }
        }
    }
}

// Define constants for square dimensions
const SQUARE_WIDTH: u16 = 4;
const SQUARE_HEIGHT: u16 = 2;

// --- TUI Drawing Functions ---
fn ui<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(8), // Captured pieces and info
                Constraint::Min(0),    // Chess board (takes remaining space)
                Constraint::Length(3), // Messages and input
            ]
            .as_ref(),
        )
        .split(f.size());

    // Captured Pieces and Info Block
    let captured_block = Block::default().borders(Borders::ALL).title(" Game Info ");

    let white_captured_chars: Vec<Span> = app
        .board
        .captured_white
        .iter()
        .map(|p| {
            Span::styled(
                p.to_char().to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
        })
        .collect();
    let black_captured_chars: Vec<Span> = app
        .board
        .captured_black
        .iter()
        .map(|p| {
            Span::styled(
                p.to_char().to_string(),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
        })
        .collect();

    let mut white_info_spans = vec![
        Span::styled("White Points: ", Style::default().fg(Color::Gray)),
        Span::styled(
            app.board.white_points.to_string(),
            Style::default().fg(Color::White),
        ),
        Span::raw("   Captured: "),
    ];
    white_info_spans.extend(white_captured_chars); // Extend with the Vec<Span>

    let mut black_info_spans = vec![
        Span::styled("Black Points: ", Style::default().fg(Color::Gray)),
        Span::styled(
            app.board.black_points.to_string(),
            Style::default().fg(Color::White),
        ),
        Span::raw("   Captured: "),
    ];
    black_info_spans.extend(black_captured_chars); // Extend with the Vec<Span>

    let info_text = vec![
        Spans::from(white_info_spans),
        Spans::from(black_info_spans),
        Spans::from(vec![
            Span::styled("Current Turn: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:?}", app.board.get_current_turn()),
                Style::default()
                    .fg(match app.board.get_current_turn() {
                        ColorChess::White => Color::White,
                        ColorChess::Black => Color::Blue,
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    let info_paragraph = Paragraph::new(info_text).block(captured_block);
    f.render_widget(info_paragraph, chunks[0]);

    // Chess Board Block
    let board_block = Block::default()
        .borders(Borders::ALL)
        .title(" Chess Board ");
    f.render_widget(board_block.clone(), chunks[1]); // Render the outer block first

    // Draw the board content manually within the board_block area
    let board_area = board_block.inner(chunks[1]);
    let board_start_col = board_area.x + 3;
    let board_start_row = board_area.y + 1;

    let ranks: Vec<usize> = if app.player_perspective == ColorChess::White {
        (0..8).rev().collect() // 8 to 1
    } else {
        (0..8).collect() // 1 to 8
    };

    for (i_idx, &r) in ranks.iter().enumerate() {
        // Rank numbers (e.g., '8', '7', ...)
        f.render_widget(
            Paragraph::new(Span::raw(format!("{}", 8 - r))),
            tui::layout::Rect::new(
                board_area.x + 1,
                board_start_row + (i_idx as u16 * SQUARE_HEIGHT) + (SQUARE_HEIGHT / 2), // Center rank label vertically
                1,
                1,
            ),
        );

        for c in 0..8 {
            let square_color = if (r + c) % 2 == 0 {
                Color::Rgb(181, 136, 99) // Dark square
            } else {
                Color::Rgb(240, 217, 181) // Light square
            };

            let mut style = Style::default().bg(square_color);

            // Highlight selected square
            if let Some(selected_sq) = app.selected_square {
                if selected_sq == (r, c) {
                    style = style
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD);
                }
            }

            // Highlight possible moves
            if app.possible_moves.contains(&(r, c)) {
                style = style
                    .bg(Color::Green)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD);
            }

            let piece_char = match app.board.squares[r][c] {
                Some(piece) => {
                    let piece_tui_color = if piece.color() == ColorChess::White {
                        Color::White
                    } else {
                        Color::Blue // Black pieces
                    };
                    Span::styled(
                        // Center the piece character within the larger square
                        format!(
                            "{:^width$}",
                            piece.to_char().to_string(),
                            width = SQUARE_WIDTH as usize
                        ),
                        Style::default()
                            .fg(piece_tui_color)
                            .add_modifier(Modifier::BOLD),
                    )
                }
                None => Span::raw(format!("{:^width$}", " ", width = SQUARE_WIDTH as usize)),
            };

            f.render_widget(
                Paragraph::new(piece_char).style(style),
                tui::layout::Rect::new(
                    board_start_col + (c as u16 * SQUARE_WIDTH),
                    board_start_row + (i_idx as u16 * SQUARE_HEIGHT),
                    SQUARE_WIDTH,
                    SQUARE_HEIGHT,
                ),
            );
        }
    }

    let file_labels: Vec<Span> = ('a'..='h')
        .map(|c| {
            Span::raw(format!(
                "{:^width$}",
                c.to_string(),
                width = SQUARE_WIDTH as usize
            ))
        })
        .collect();
    f.render_widget(
        Paragraph::new(Spans::from(file_labels)),
        tui::layout::Rect::new(
            board_start_col,
            board_start_row + (8 * SQUARE_HEIGHT),
            8 * SQUARE_WIDTH,
            1,
        ),
    );

    // Messages and Input Block
    let message_block = Block::default().borders(Borders::ALL).title(" Messages ");
    let message_paragraph = Paragraph::new(app.message.as_str()).block(message_block);
    f.render_widget(message_paragraph, chunks[2]);
}

// --- Main Game Loop ---
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    // Enable mouse capture
    execute!(stdout, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let tick_rate = Duration::from_millis(250); // For UI refresh
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                CrosstermEvent::Key(key) => {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        break; // Quit
                    }
                }
                CrosstermEvent::Mouse(mouse_event) => {
                    if mouse_event.kind == MouseEventKind::Down(event::MouseButton::Left) {
                        app.handle_mouse_click(mouse_event.column, mouse_event.row);
                    }
                }
                CrosstermEvent::Resize(_, _) => {
                    // TODO:
                    // Handle terminal resize events
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.game_over_message.is_some() {
            if event::poll(Duration::from_millis(100))? {
                if let CrosstermEvent::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        break;
                    }
                }
            }
        }
    }

    // Restore terminal
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    // Disable mouse capture
    execute!(terminal.backend_mut(), event::DisableMouseCapture)?;
    disable_raw_mode()?;

    Ok(())
}
