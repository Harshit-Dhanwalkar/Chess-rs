use std::io;
use std::option::Option;
use std::vec::Vec;

#[derive(Clone)]
struct Board {
    squares: [[Option<Piece>; 8]; 8],
    captured_white: Vec<Piece>,
    captured_black: Vec<Piece>,
    current_turn: Color,
    white_points: u32,
    black_points: u32,
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
enum Color {
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
const WHITE: u8 = 0b0000;
const BLACK: u8 = 0b1000;

impl Piece {
    // Constructor
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        let type_bits = match piece_type {
            PieceType::Pawn => PAWN,
            PieceType::Knight => KNIGHT,
            PieceType::Bishop => BISHOP,
            PieceType::Rook => ROOK,
            PieceType::Queen => QUEEN,
            PieceType::King => KING,
        };

        let color_bit = match color {
            Color::White => WHITE,
            Color::Black => BLACK,
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

    pub fn color(&self) -> Color {
        if (self.0 & BLACK) != 0 {
            Color::Black
        } else {
            Color::White
        }
    }

    pub fn is_color(&self, color: Color) -> bool {
        self.color() == color
    }

    pub fn is_type(&self, piece_type: PieceType) -> bool {
        self.piece_type() == piece_type
    }

    fn to_char(&self) -> String {
        let symbol = match self.piece_type() {
            PieceType::King => {
                if self.color() == Color::White {
                    '♚'
                } else {
                    '♚'
                }
            }
            PieceType::Queen => {
                if self.color() == Color::White {
                    '♛'
                } else {
                    '♛'
                }
            }
            PieceType::Rook => {
                if self.color() == Color::White {
                    '♜'
                } else {
                    '♜'
                }
            }
            PieceType::Bishop => {
                if self.color() == Color::White {
                    '♝'
                } else {
                    '♝'
                }
            }
            PieceType::Knight => {
                if self.color() == Color::White {
                    '♞'
                } else {
                    '♞'
                }
            }
            PieceType::Pawn => {
                if self.color() == Color::White {
                    '♟'
                } else {
                    '♟'
                }
            }
        };
        if self.color() == Color::White {
            format!("\x1b[1;97m{}\x1b[0m", symbol)
        } else {
            format!("\x1b[1;34m{}\x1b[0m", symbol)
        }
    }

    fn points(&self) -> u32 {
        match self.piece_type() {
            PieceType::Pawn => 1,
            PieceType::Knight | PieceType::Bishop => 3,
            PieceType::Rook => 5,
            PieceType::Queen => 9,
            PieceType::King => 0,
        }
    }
}

impl Board {
    fn new() -> Board {
        let mut squares = [[None; 8]; 8];
        for i in 0..8 {
            squares[1][i] = Some(Piece::new(PieceType::Pawn, Color::White));
            squares[6][i] = Some(Piece::new(PieceType::Pawn, Color::Black));
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
            squares[0][i] = Some(Piece::new(piece_type, Color::White));
            squares[7][i] = Some(Piece::new(piece_type, Color::Black));
        }

        Board {
            squares,
            captured_white: Vec::new(),
            captured_black: Vec::new(),
            current_turn: Color::White,
            white_points: 0,
            black_points: 0,
        }
    }

    fn choose_player_color() -> Color {
        loop {
            let white_king = Piece::new(PieceType::King, Color::White).to_char();
            let black_king = Piece::new(PieceType::King, Color::Black).to_char();

            println!(
                "Choose your color:\n White{} / Black{}\n(Enter W or B):",
                white_king, black_king
            );

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            match input.trim().to_uppercase().as_str() {
                "W" => return Color::White,
                "B" => return Color::Black,
                _ => println!("Invalid input. Please enter 'W' or 'B'."),
            }
        }
    }

    fn print_board(&self, highlights: &[(usize, usize)], player_perspective: Color) {
        println!("   a b c d e f g h");
        println!("  ┌────────────────┐");
        let rows: Vec<usize> = if player_perspective == Color::White {
            (0..8).collect()
        } else {
            (0..8).rev().collect()
        };

        for &i in &rows {
            print!("{} │", (b'8' - i as u8) as char);
            for j in 0..8 {
                if highlights.contains(&(i, j)) {
                    print!("* ");
                } else {
                    match self.squares[i][j] {
                        Some(piece) => print!("{} ", piece.to_char()),
                        None => print!(". "),
                    }
                }
            }
            println!("│");
        }
        println!("  └────────────────┘");
        println!("   a b c d e f g h");
    }

    fn is_valid_move(&self, start: (usize, usize), end: (usize, usize), color: Color) -> bool {
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
        if let Some(captured) = self.squares[end.0][end.1].take() {
            if captured.color() == Color::White {
                self.captured_white.push(captured);
                self.white_points += captured.points();
            } else {
                self.captured_black.push(captured);
                self.black_points += captured.points();
            }
        }
        if let Some(piece) = self.squares[start.0][start.1].take() {
            self.squares[end.0][end.1] = Some(piece);
        }
    }

    fn print_captured_pieces(&self) {
        let white_captured_chars: Vec<String> =
            self.captured_white.iter().map(|p| p.to_char()).collect();
        let black_captured_chars: Vec<String> =
            self.captured_black.iter().map(|p| p.to_char()).collect();

        // Join the captured pieces with a space for better readability
        let white_captured_display = if white_captured_chars.is_empty() {
            String::from("")
        } else {
            white_captured_chars.join(" ")
        };

        let black_captured_display = if black_captured_chars.is_empty() {
            String::from("")
        } else {
            black_captured_chars.join(" ")
        };

        println!("┌──────────────────────────┬───────────────────────────────────┐");
        println!("│        POINTS            │         CAPTURED PIECES           │");
        println!("├──────────────────────────┼───────────────────────────────────┤");
        println!(
            "│ White: {:<17} │ White: {:<27}│",
            self.white_points, white_captured_display
        );
        println!(
            "│ Black: {:<17} │ Black: {:<27}│",
            self.black_points, black_captured_display
        );
        println!("└──────────────────────────┴───────────────────────────────────┘");
    }

    fn get_all_moves(&self, color: Color) -> Vec<((usize, usize), (usize, usize))> {
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

    fn is_valid_pawn_move(&self, start: (usize, usize), end: (usize, usize), color: Color) -> bool {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        if color == Color::White {
            if start_x < 7 && start_x + 1 == end_x && start_y == end_y {
                return self.squares[end_x][end_y].is_none();
            }
            if start_x == 1 && end_x == 3 && start_y == end_y {
                return self.squares[2][end_y].is_none() && self.squares[end_x][end_y].is_none();
            }
            if start_x < 7 && start_x + 1 == end_x && (start_y as isize - end_y as isize).abs() == 1
            {
                if let Some(piece) = &self.squares[end_x][end_y] {
                    if piece.color() == Color::Black {
                        return true;
                    }
                }
            }
        } else {
            if start_x > 0 && start_x - 1 == end_x && start_y == end_y {
                return self.squares[end_x][end_y].is_none();
            }
            if start_x == 6 && end_x == 4 && start_y == end_y {
                return self.squares[5][end_y].is_none() && self.squares[end_x][end_y].is_none();
            }
            if start_x > 0 && start_x - 1 == end_x && (start_y as isize - end_y as isize).abs() == 1
            {
                if let Some(piece) = &self.squares[end_x][end_y] {
                    if piece.color() == Color::White {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_valid_knight_move(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        color: Color,
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

    fn is_valid_bishop_move(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        color: Color,
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

    fn is_valid_rook_move(&self, start: (usize, usize), end: (usize, usize), color: Color) -> bool {
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

    fn is_valid_queen_move(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        color: Color,
    ) -> bool {
        self.is_valid_rook_move(start, end, color) || self.is_valid_bishop_move(start, end, color)
    }

    fn is_valid_king_move(&self, start: (usize, usize), end: (usize, usize), color: Color) -> bool {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

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

    fn find_king(&self, color: Color) -> Option<(usize, usize)> {
        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = &self.squares[x][y] {
                    if piece.is_type(PieceType::King) && piece.is_color(color) {
                        return Some((x, y));
                    }
                }
            }
        }
        println!("Error: King of {:?} not found!", color);
        None
    }

    fn is_in_check(&self, color: Color) -> bool {
        println!("Checking if the king of {:?} is in check", color);
        let king_position = match self.find_king(color) {
            Some(pos) => pos,
            None => return false,
        };

        let opponent_color = if color == Color::White {
            Color::Black
        } else {
            Color::White
        };

        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = &self.squares[x][y] {
                    if piece.color() == opponent_color {
                        if self.is_valid_move((x, y), king_position, opponent_color) {
                            println!(
                                "King of {:?} is in check! Attacked by {:?} at ({}, {})",
                                color,
                                piece.piece_type(),
                                x,
                                y
                            );
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn is_checkmate(&mut self, color: Color) -> bool {
        if self.find_king(color).is_none() {
            return true;
        }

        if !self.is_in_check(color) {
            return false;
        }

        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = self.squares[x][y].clone() {
                    if piece.color() == color {
                        for new_x in 0..8 {
                            for new_y in 0..8 {
                                if self.is_valid_move((x, y), (new_x, new_y), color) {
                                    let temp_piece = self.squares[new_x][new_y].clone();
                                    self.squares[new_x][new_y] = Some(piece);
                                    self.squares[x][y] = None;

                                    let is_still_in_check = self.is_in_check(color);

                                    self.squares[x][y] = Some(piece);
                                    self.squares[new_x][new_y] = temp_piece;

                                    if !is_still_in_check {
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        true
    }

    // TODO: implement check_after_move
    //
    // fn is_in_check_after_move(&self, start: (usize, usize), end: (usize, usize), color: Color) -> bool {
    //     let mut board_copy = self.clone();
    //     board_copy.move_piece(start, end);
    //     board_copy.is_in_check(color)
    // }

    // TODO: implement stalemate
    //
    // fn is_stalemate(&self, color: Color) -> bool {
    //     if self.is_in_check(color) {
    //         return false; // Can't be stalemate if the king is in check
    //     }
    //     // Check if there are any legal moves for the player
    //     for x in 0..8 {
    //         for y in 0..8 {
    //             if let Some(piece) = &self.squares[x][y] {
    //                 if piece.color == color {
    //                     // Try moving each piece to any valid square
    //                     for new_x in 0..8 {
    //                         for new_y in 0..8 {
    //                             if self.is_valid_move((x, y), (new_x, new_y), color) {
    //                                 return false; // There is a move left for the player
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     true // No moves left, it's stalemate
    // }

    fn has_king(&self, color: Color) -> bool {
        for row in &self.squares {
            for square in row {
                if let Some(piece) = square {
                    if piece.is_type(PieceType::King) && piece.is_color(color) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_game_over(&mut self, color: Color) -> bool {
        if self.is_checkmate(color) || self.get_all_moves(color).is_empty() {
            return true;
        }
        // TODO: add other checks here (stalemate, is_in_check_after_move, insufficient material)
        // if self.is_checkmate(color) || self.is_stalemate(color) || self.is_in_check_after_move || self.get_all_moves(color).is_empty() {
        //     return true; // Game is over if checkmate or no valid moves left
        // }
        false
    }

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
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
    }

    fn get_current_turn(&self) -> Color {
        self.current_turn
    }

    // TODO: to implement minimax for AI/automation
    // // evaluate the board (material balance)
    // fn evaluate(&self) -> i32 {
    //     let mut score = 0;
    //
    //     // iterate over the entire board
    //     for row in &self.squares {
    //         for square in row {
    //             if let Some(piece) = square {
    //                 let piece_value = match piece.piece_type {
    //                     PieceType::Pawn => 1,
    //                     PieceType::Knight => 3,
    //                     PieceType::Bishop => 3,
    //                     PieceType::Rook => 5,
    //                     PieceType::Queen => 9,
    //                     PieceType::King => 1000,
    //                 };
    //                 // add score for white pieces, subtract for black pieces
    //                 score += if piece.color == Color::White {
    //                     piece_value
    //                 } else {
    //                     -piece_value
    //                 };
    //             }
    //         }
    //     }
    //     score
    // }
    // // minimax function for decision-making (basic ai)
    // fn minimax(Board: &Board, depth: usize, is_maximizing: bool) -> i32 {
    //     if depth == 0 || Board.is_game_over(Color::White) || Board.is_game_over(Color::Black) {
    //         return Board.evaluate();
    //     }
    //
    //     if is_maximizing {
    //         let mut max_eval = i32::MIN;  // Initialize to the smallest possible value
    //         let moves = Board.get_all_moves(Color::White); // white's turn
    //         for mv in moves {
    //             let mut Board_copy = Board.clone();
    //             Board_copy.move_piece(mv);
    //             let eval = Board::minimax(&Board_copy, depth - 1, false); // Recursively call minimax
    //             max_eval = std::cmp::max(max_eval, eval); // Update max_eval with the maximum value
    //         }
    //         max_eval // Return the best evaluation for maximizing player
    //     } else {
    //         let mut min_eval = i32::MAX; // Initialize to the largest possible value
    //         let moves = Board.get_all_moves(Color::Black); // black's turn
    //         for mv in moves {
    //             let mut Board_copy = Board.clone();
    //             Board_copy.move_piece(mv);
    //             let eval = Board::minimax(&Board_copy, depth - 1, true);// Recursively call minimax
    //             min_eval = std::cmp::min(min_eval, eval); // Update min_eval with the minimum value
    //         }
    //         min_eval
    //     }
    // }
}

fn main() {
    let mut board = Board::new();
    let player_color = Board::choose_player_color();

    println!(
        "White has {} valid moves.",
        board.get_all_moves(Color::White).len()
    );
    println!(
        "Black has {} valid moves.",
        board.get_all_moves(Color::Black).len()
    );

    println!("Is White in check? {}", board.is_in_check(Color::White));
    println!("Is Black in check? {}", board.is_in_check(Color::Black));

    println!(
        "Is the game over for White? {}",
        board.is_game_over(Color::White)
    );
    println!(
        "Is the game over for Black? {}",
        board.is_game_over(Color::Black)
    );

    while !board.is_game_over(board.get_current_turn()) {
        let highlights = vec![];
        board.print_board(&highlights, player_color);
        board.print_captured_pieces();

        println!(
            "Player {:?}'s turn (enter 'q' to quit)",
            board.get_current_turn()
        );
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let trimmed_input = input.trim();
        if trimmed_input == "q" || trimmed_input == "\x1B" {
            println!("Game exited by user.");
            return;
        }

        if trimmed_input.len() != 4 {
            println!("Invalid move format. Use 'e2e4' or 'q'/'ESC' to quit.");
            continue;
        }

        let start = board.parse_move(&trimmed_input[0..2]);
        let end = board.parse_move(&trimmed_input[2..4]);

        if let (Some((start_x, start_y)), Some((end_x, end_y))) = (start, end) {
            println!(
                "Parsed start: ({}, {}), end: ({}, {})",
                start_x, start_y, end_x, end_y
            );
            if board.is_valid_move((start_x, start_y), (end_x, end_y), board.get_current_turn()) {
                board.move_piece((start_x, start_y), (end_x, end_y));
                if board.is_checkmate(board.get_current_turn()) {
                    board.print_board(&vec![], player_color);
                    println!(
                        "Checkmate! {:?} wins.",
                        match board.get_current_turn() {
                            Color::White => Color::Black,
                            Color::Black => Color::White,
                        }
                    );
                    return;
                }
                board.switch_turn();
            } else {
                println!("invalid move, try again.");
            }
        } else {
            println!("Invalid move format or out-of-bound coordinates.");
        }
    }

    if !board.has_king(Color::White) {
        board.print_board(&vec![], player_color);
        println!("Game over! Black wins. White's king has been captured.");
        return;
    }
    if !board.has_king(Color::Black) {
        board.print_board(&vec![], player_color);
        println!("Game over! White wins. Black's king has been captured.");
        return;
    }
}
