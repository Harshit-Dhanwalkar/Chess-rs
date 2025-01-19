use std::io;
use std::option::Option;
use std::vec::Vec;

#[derive(Clone)]
struct Board {
    squares: [[Option<Piece>; 8]; 8],
    // To store captured pieces
    captured_white: Vec<Piece>,
    captured_black: Vec<Piece>,
    // to get current turn
    current_turn: Color,
    // for point counter/tracker
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

#[derive(Clone, Copy)]
struct Piece {
    piece_type: PieceType,
    color: Color,
}

impl Piece {
    fn to_char(&self) -> String {
        let symbol = match self.piece_type {
            PieceType::King => {
                if self.color == Color::White {
                    '♔'
                } else {
                    '♚'
                }
            }
            PieceType::Queen => {
                if self.color == Color::White {
                    '♕'
                } else {
                    '♛'
                }
            }
            PieceType::Rook => {
                if self.color == Color::White {
                    '♖'
                } else {
                    '♜'
                }
            }
            PieceType::Bishop => {
                if self.color == Color::White {
                    '♗'
                } else {
                    '♝'
                }
            }
            PieceType::Knight => {
                if self.color == Color::White {
                    '♘'
                } else {
                    '♞'
                }
            }
            PieceType::Pawn => {
                if self.color == Color::White {
                    '♙'
                } else {
                    '♟'
                }
            }
        };
        if self.color == Color::White {
            format!("\x1b[1;97m{}\x1b[0m", symbol) // White pieces in bold
        } else {
            format!("\x1b[1;34m{}\x1b[0m", symbol) // Black pieces in blue
        }
    }
    fn points(&self) -> u32 {
        match self.piece_type {
            PieceType::Pawn => 1,
            PieceType::Knight | PieceType::Bishop => 3,
            PieceType::Rook => 5,
            PieceType::Queen => 9,
            PieceType::King => 0, // King has no point value for captures
        }
    }
}

impl Board {
    // Constructor for Board
    fn new() -> Board {
        let mut squares = [[None; 8]; 8]; // Initialize empty squares with None
                                          // Initialize pawns
        for i in 0..8 {
            squares[1][i] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: Color::White,
            });
            squares[6][i] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: Color::Black,
            });
        }
        // Initialize other pieces
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
            squares[0][i] = Some(Piece {
                piece_type,
                color: Color::White,
            });
            squares[7][i] = Some(Piece {
                piece_type,
                color: Color::Black,
            });
        }
        // Initialize the board with an empty captured pieces array
        Board {
            squares,
            captured_white: Vec::new(),
            captured_black: Vec::new(),

            current_turn: Color::White, // White starts the game

            white_points: 0,
            black_points: 0,
        }
    }

    fn print_board(&self, highlights: &[(usize, usize)]) {
        //fn print_board(&self) {
        println!("   a b c d e f g h");
        println!("  ┌────────────────┐");
        for (i, row) in self.squares.iter().enumerate() {
            //print!("{} │", 8 - i);
            print!("{} │", (b'8' - i as u8) as char);
            for (j, square) in row.iter().enumerate() {
                if highlights.contains(&(i, j)) {
                    print!("* "); // Highlighted move
                } else {
                    match square {
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

    // general move validation for all pieces
    fn is_valid_move(&self, start: (usize, usize), end: (usize, usize), color: Color) -> bool {
        // println!(
        //      "Checking move validity for color {:?}: ({}, {}) -> ({}, {})",
        //      color, start.0, start.1, end.0, end.1
        // );
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        if start == end || end_x >= 8 || end_y >= 8 {
            return false; // a move to the same position is not allowed
        }
        if let Some(piece) = &self.squares[start_x][start_y] {
            if piece.color != color {
                return false; // cannot move an opponent's piece
            }
            match piece.piece_type {
                PieceType::Pawn => self.is_valid_pawn_move(start, end, color),
                PieceType::Knight => self.is_valid_knight_move(start, end, color),
                PieceType::Bishop => self.is_valid_bishop_move(start, end, color),
                PieceType::Rook => self.is_valid_rook_move(start, end, color),
                PieceType::Queen => self.is_valid_queen_move(start, end, color),
                PieceType::King => self.is_valid_king_move(start, end, color),
            }
        } else {
            false // no piece to move
        }
    }

    // Move a piece from the start to the end position
    fn move_piece(&mut self, start: (usize, usize), end: (usize, usize)) {
        if let Some(captured) = self.squares[end.0][end.1].take() {
            // Add captured piece to the list
            if captured.color == Color::White {
                self.captured_white.push(captured); // Add white piece to the captured list
                self.white_points += captured.points(); // Add points for White
            } else {
                self.captured_black.push(captured); // Add black piece to the captured list
                self.black_points += captured.points(); // Add points for Black
            }
        }
        // Move the piece from start to end
        if let Some(piece) = self.squares[start.0][start.1].take() {
            self.squares[end.0][end.1] = Some(piece);
        }
    }

    fn print_captured_pieces(&self) {
        // Convert captured pieces to a string representation of their characters
        let white_captured: String = self.captured_white.iter().map(|p| p.to_char()).collect();
        let black_captured: String = self.captured_black.iter().map(|p| p.to_char()).collect();

        println!("┌──────────────────────────┬─────────────────────────────┐");
        println!(
            "│ {:<10}               │ {:<13}               │",
            "Points ", "Captured pieces"
        );
        println!("├──────────────────────────┼─────────────────────────────┤");
        println!(
            "│ {:<10}               │ White: {:<13}        │",
            self.white_points, white_captured
        );
        println!(
            "│ {:<10}               │ Black: {:<13}        │",
            self.black_points, black_captured
        );
        println!("└──────────────────────────┴─────────────────────────────┘");
    }

    // check if the game is over (checkmate or stalemate)
    fn get_all_moves(&self, color: Color) -> Vec<((usize, usize), (usize, usize))> {
        let mut moves = Vec::new();
        for start_x in 0..8 {
            for start_y in 0..8 {
                if let Some(piece) = &self.squares[start_x][start_y] {
                    if piece.color == color {
                        // println!("Checking moves for piece at ({}, {})", start_x, start_y);
                        for end_x in 0..8 {
                            for end_y in 0..8 {
                                if self.is_valid_move((start_x, start_y), (end_x, end_y), color) {
                                    // println!("Valid move: ({}, {}) -> ({}, {})", start_x, start_y, end_x, end_y);
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
            // Single step forward for White pawns
            if start_x < 7 && start_x + 1 == end_x && start_y == end_y {
                //println!("White pawn single step: ({},{}) -> ({},{})", start_x, start_y, end_x, end_y);
                return self.squares[end_x][end_y].is_none();
            }
            // Double step forward from starting position
            if start_x == 1 && end_x == 3 && start_y == end_y {
                // println!("White pawn double step: ({},{}) -> ({},{})", start_x, start_y, end_x, end_y);
                return self.squares[2][end_y].is_none() && self.squares[end_x][end_y].is_none();
            }
            // Diagonal capture for White pawns
            if start_x < 7 && start_x + 1 == end_x && (start_y as isize - end_y as isize).abs() == 1
            {
                if let Some(piece) = &self.squares[end_x][end_y] {
                    if piece.color == Color::Black {
                        // println!("White pawn diagonal capture: ({},{}) -> ({},{})", start_x, start_y, end_x, end_y);
                        return true;
                    }
                }
            }
        } else {
            // Single step forward for Black pawns
            if start_x > 0 && start_x - 1 == end_x && start_y == end_y {
                // println!("Black pawn single step: ({},{}) -> ({},{})", start_x, start_y, end_x, end_y);
                return self.squares[end_x][end_y].is_none();
            }
            // Double step forward from starting position
            if start_x == 6 && end_x == 4 && start_y == end_y {
                // println!("Black pawn double step: ({},{}) -> ({},{})", start_x, start_y, end_x, end_y);
                return self.squares[5][end_y].is_none() && self.squares[end_x][end_y].is_none();
            }
            // Diagonal capture for Black pawns
            if start_x > 0 && start_x - 1 == end_x && (start_y as isize - end_y as isize).abs() == 1
            {
                if let Some(piece) = &self.squares[end_x][end_y] {
                    if piece.color == Color::White {
                        // println!("Black pawn diagonal capture: ({},{}) -> ({},{})", start_x, start_y, end_x, end_y);
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

        // knight move/capture
        if (dx == 2 && dy == 1) || (dx == 1 && dy == 2) {
            return self.squares[end_x][end_y].is_none()
                || self.squares[end_x][end_y].unwrap().color != color; // capture an opponent's piece
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

        // bishop moves diagonally, so the absolute difference in x and y must be equal
        if (start_x as isize - end_x as isize).abs() != (start_y as isize - end_y as isize).abs() {
            return false;
        }

        // check for clear path (no pieces between start and end)
        let dx = if end_x > start_x { 1 } else { -1 };
        let dy = if end_y > start_y { 1 } else { -1 };

        let mut x = start_x as isize + dx;
        let mut y = start_y as isize + dy;

        while (x != end_x as isize) && (y != end_y as isize) {
            if self.squares[x as usize][y as usize].is_some() {
                return false; // a piece is blocking the path
            }
            x += dx;
            y += dy;
        }

        // return self.squares[end_x][end_y].is_none() || self.squares[end_x][end_y].unwrap().color != color;
        self.squares[end_x][end_y].is_none()
            || self.squares[end_x][end_y].map_or(false, |p| p.color != color)
    }

    fn is_valid_rook_move(&self, start: (usize, usize), end: (usize, usize), color: Color) -> bool {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        // Rook can only move horizontally or vertically
        if start_x != end_x && start_y != end_y {
            return false; // Not a valid rook move
        }

        // Check if path is clear (no pieces blocking the rook's movement)
        if start_x == end_x {
            let range = if start_y < end_y {
                start_y + 1..end_y
            } else {
                end_y + 1..start_y
            };
            for y in range {
                if self.squares[start_x][y].is_some() {
                    return false; // Path blocked
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
                    return false; // Path blocked
                }
            }
        }

        // Check if destination is empty or occupied by an opponent's piece
        if let Some(piece) = &self.squares[end_x][end_y] {
            return piece.color != color; // Cannot land on a piece of the same color
        }

        true
    }

    // validity check for queen movement (combines bishop and rook movement)
    fn is_valid_queen_move(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        color: Color,
    ) -> bool {
        self.is_valid_rook_move(start, end, color) || self.is_valid_bishop_move(start, end, color)
    }

    // validity check for king movement
    fn is_valid_king_move(&self, start: (usize, usize), end: (usize, usize), color: Color) -> bool {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        let dx = (end_x as isize - start_x as isize).abs();
        let dy = (end_y as isize - start_y as isize).abs();

        // kings move one square in any direction
        // if (end_x as isize - start_x as isize).abs() <= 1 && (end_y as isize - start_y as isize).abs() <= 1 {
        //     return self.squares[end_x][end_y].is_none() || self.squares[end_x][end_y].unwrap().color != color;
        // }
        if dx <= 1 && dy <= 1 {
            // Check if the destination is empty or occupied by an opponent's piece
            // self.squares[end_x][end_y].is_none()
            //     || self.squares[end_x][end_y].map_or(false, |p| p.color != color)
            // } else {
            //     false
            // }
            if let Some(piece) = &self.squares[end_x][end_y] {
                piece.color != color
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
                    if piece.piece_type == PieceType::King && piece.color == color {
                        return Some((x, y));
                    }
                }
            }
        }
        println!("Error: King of {:?} not found!", color);
        None
    }

    // check if a king is in check (attacked by an opposing piece)
    fn is_in_check(&self, color: Color) -> bool {
        println!("Checking if the king of {:?} is in check", color);
        let king_position = match self.find_king(color) {
            Some(pos) => pos,
            None => return false, // If the king is not found, can't be in check
        };

        // Loop through the opponent's pieces and check if any can attack the king
        let opponent_color = if color == Color::White {
            Color::Black
        } else {
            Color::White
        };
        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = &self.squares[x][y] {
                    if piece.color == opponent_color {
                        // Check if the opponent's piece can attack the king's position
                        if self.is_valid_move((x, y), king_position, opponent_color) {
                            // Print when the king is in check
                            println!(
                                "King of {:?} is in check! Attacked by {:?} at ({}, {})",
                                color, piece.piece_type, x, y
                            );
                            return true; // King is in check
                        }
                    }
                }
            }
        }
        false // King is not in check
    }

    // fn is_checkmate(&mut self, color: Color) -> bool {
    //     if !self.is_in_check(color) {
    //         return false;
    //     }
    //
    //     for x in 0..8 {
    //         for y in 0..8 {
    //             if let Some(piece) = self.squares[x][y].clone() {
    //                 // Clone the piece to avoid borrowing
    //                 if piece.color == color {
    //                     for new_x in 0..8 {
    //                         for new_y in 0..8 {
    //                             if self.is_valid_move((x, y), (new_x, new_y), color) {
    //                                 let temp_piece = self.squares[new_x][new_y].clone();
    //                                 self.squares[new_x][new_y] = Some(piece.clone());
    //                                 self.squares[x][y] = None;
    //
    //                                 let is_still_in_check = self.is_in_check(color);
    //
    //                                 // Restore the board state
    //                                 self.squares[x][y] = Some(piece);
    //                                 self.squares[new_x][new_y] = temp_piece;
    //
    //                                 if !is_still_in_check {
    //                                     return false;
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //
    //     true
    // }
    fn is_checkmate(&mut self, color: Color) -> bool {
        // Check if the king is missing (captured)
        if self.find_king(color).is_none() {
            // If the king is missing, it's checkmate (game over)
            return true;
        }

        // If the king is in check, check if any valid moves can get the king out of check
        if !self.is_in_check(color) {
            return false;
        }

        // Iterate over all squares to find the color's pieces
        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = self.squares[x][y].clone() {
                    // Clone the piece to avoid borrowing
                    if piece.color == color {
                        // Check all possible moves for this piece
                        for new_x in 0..8 {
                            for new_y in 0..8 {
                                if self.is_valid_move((x, y), (new_x, new_y), color) {
                                    let temp_piece = self.squares[new_x][new_y].clone();
                                    self.squares[new_x][new_y] = Some(piece.clone());
                                    self.squares[x][y] = None;

                                    // Check if the king is still in check after the move
                                    let is_still_in_check = self.is_in_check(color);

                                    // Restore the board state
                                    self.squares[x][y] = Some(piece);
                                    self.squares[new_x][new_y] = temp_piece;

                                    // If the move doesn't result in the king being in check, it's not checkmate
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

        // If no valid move can escape check, it's checkmate
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

    // Check if a given color's king is still on the board
    fn has_king(&self, color: Color) -> bool {
        for row in &self.squares {
            for square in row {
                if let Some(piece) = square {
                    if piece.piece_type == PieceType::King && piece.color == color {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_game_over(&mut self, color: Color) -> bool {
        if self.is_checkmate(color) || self.get_all_moves(color).is_empty() {
            return true; // Game is over if checkmate or no valid moves left
        }
        // TODO: add other checks here (stalemate, is_in_check_after_move, insufficient material)
        // if self.is_checkmate(color) || self.is_stalemate(color) || self.is_in_check_after_move || self.get_all_moves(color).is_empty() {
        //     return true; // Game is over if checkmate or no valid moves left
        // }
        false
    }

    // // check if a player has valid moves
    // fn has_valid_moves(&self, color: Color) -> bool {
    //     for x in 0..8 {
    //         for y in 0..8 {
    //             if let Some(piece) = &self.squares[x][y] {
    //                 if piece.color == color {
    //                     for dx in 0..8 {
    //                         for dy in 0..8 {
    //                             if self.is_valid_move((x, y), (dx, dy), color) {
    //                                 return true;
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     false
    // }

    fn parse_move(&self, move_str: &str) -> Option<(usize, usize)> {
        if move_str.len() != 2 {
            return None; // Input must be exactly two characters
        }

        let chars: Vec<char> = move_str.chars().collect();
        let col = chars[0].to_ascii_lowercase(); // Column letter (a-h)
        let row = chars[1]; // Row number (1-8)

        if !('a'..='h').contains(&col) || !('1'..='8').contains(&row) {
            return None; // Invalid input
        }

        let col_index = (col as usize) - ('a' as usize); // Convert column to index (a=0, b=1, ...)
        let row_index = 8 - (row.to_digit(10)? as usize); // Convert row to index (8=0, 7=1, ...)

        Some((row_index, col_index))
    }

    // Switch the turn between players
    fn switch_turn(&mut self) {
        self.current_turn = match self.current_turn {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
    }

    // Get the current turn
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

// fn clear_screen() {
//     print!("\x1b[2J\x1b[H");
// }

fn main() {
    let mut board = Board::new();

    // let white_moves = board.get_all_moves(Color::White);
    // let black_moves = board.get_all_moves(Color::Black);

    //let mut current_player = Color::White; // White starts the game // this now in Board

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
        board.print_board(&highlights);
        board.print_captured_pieces();

        //println!("enter your move (e.g., e2e4):");
        println!("player {:?}'s turn", board.get_current_turn());
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        if input.trim().len() != 4 {
            println!("Invalid move format. Use 'e2e4'.");
            continue;
        }

        // Parse start and end positions
        let start = board.parse_move(&input[0..2]);
        let end = board.parse_move(&input[2..4]);

        if let (Some((start_x, start_y)), Some((end_x, end_y))) = (start, end) {
            println!(
                "Parsed start: ({}, {}), end: ({}, {})",
                start_x, start_y, end_x, end_y
            );
            // Check if the move is valid
            if board.is_valid_move((start_x, start_y), (end_x, end_y), board.get_current_turn()) {
                // Make the move
                board.move_piece((start_x, start_y), (end_x, end_y));
                // Check for checkmate
                if board.is_checkmate(board.get_current_turn()) {
                    board.print_board(&vec![]);
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
                // current_player = board.get_current_turn(); // Update the variable if still needed
            } else {
                println!("invalid move, try again.");
            }
        } else {
            println!("Invalid move format or out-of-bound coordinates.");
        }
    }

    // Game Over
    if !board.has_king(Color::White) {
        board.print_board(&vec![]);
        println!("Game over! Black wins. White's king has been captured.");
        return;
    }
    if !board.has_king(Color::Black) {
        board.print_board(&vec![]);
        println!("Game over! White wins. Black's king has been captured.");
        return;
    }
}
