use std::io;
use std::option::Option; 
use std::vec::Vec;

#[derive(Clone)]
struct Board {
    squares: [[Option<Piece>; 8]; 8],
}

#[derive(Clone, Copy, PartialEq)]
enum PieceType {
    Pawn, Knight, Bishop, Rook, Queen, King,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Color {
    White, Black,
}

#[derive(Clone, Copy)]
struct Piece {
    piece_type: PieceType,
    color: Color,
}

impl Piece {
    fn to_char(&self) -> char {
        match self.piece_type {
            PieceType::Pawn => if self.color == Color::White { 'P' } else { 'p' },
            PieceType::Knight => if self.color == Color::White { 'N' } else { 'n' },
            PieceType::Bishop => if self.color == Color::White { 'B' } else { 'b' },
            PieceType::Rook => if self.color == Color::White { 'R' } else { 'r' },
            PieceType::Queen => if self.color == Color::White { 'Q' } else { 'q' },
            PieceType::King => if self.color == Color::White { 'K' } else { 'k' },
        }
    }
}

impl Board {
    fn new() -> Board {
        let mut squares = [[None; 8]; 8]; // Initialize empty squares with None

        // Initialize pawns
        for i in 0..8 {
            squares[1][i] = Some(Piece { piece_type: PieceType::Pawn, color: Color::White });
            squares[6][i] = Some(Piece { piece_type: PieceType::Pawn, color: Color::Black });
        }

        // Initialize other pieces
        let back_rank = [
            PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::Queen,
            PieceType::King, PieceType::Bishop, PieceType::Knight, PieceType::Rook,
        ];
        for (i, &piece_type) in back_rank.iter().enumerate() {
            squares[0][i] = Some(Piece { piece_type, color: Color::White });
            squares[7][i] = Some(Piece { piece_type, color: Color::Black });
        }

        Board { squares }
    }

    fn print_board(&self) {
        println!("12345678");
        for row in self.squares.iter() {
            for square in row.iter() {
                match square {
                      Some(piece) => print!("{}", piece.to_char()),
                      None => print!("."),
                }
            }
            println!();
        }
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

    fn move_piece(&mut self, start: (usize, usize), end: (usize, usize)) {
        if let Some(piece) = self.squares[start.0][start.1].take() {
            self.squares[end.0][end.1] = Some(piece);
        }
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

    fn is_game_over(&self, color: Color) -> bool {
            self.get_all_moves(color).is_empty()
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
            if start_x < 7 && start_x + 1 == end_x && (start_y as isize - end_y as isize).abs() == 1 {
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
            if start_x > 0 && start_x - 1 == end_x && (start_y as isize - end_y as isize).abs() == 1 {
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

    fn is_valid_knight_move(&self, start: (usize, usize), end: (usize, usize), color: Color) -> bool {
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

    fn is_valid_bishop_move(&self, start: (usize, usize), end: (usize, usize), color: Color) -> bool {
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
        self.squares[end_x][end_y].is_none() || self.squares[end_x][end_y].map_or(false, |p| p.color != color)
    }

    // fn is_valid_rook_move(&self, start: (usize, usize), end: (usize, usize), color: Color) -> bool {
    //     let (start_x, start_y) = start;
    //     let (end_x, end_y) = end;
    //
    //     // rook moves along rows or columns
    //     if start_x != end_x && start_y != end_y {
    //         return false; // rook can't move diagonally
    //     }
    //
    //     // check for clear path (no pieces between start and end)
    //     if start_x == end_x {
    //         let min_y = start_y.min(end_y);
    //         let max_y = start_y.max(end_y);
    //         for y in (min_y + 1)..max_y {
    //             if self.squares[start_x][y].is_some() {
    //                 return false; // a piece is blocking the path
    //             }
    //         }
    //     } else {
    //         let min_x = start_x.min(end_x);
    //         let max_x = start_x.max(end_x);
    //         for x in (min_x + 1)..max_x {
    //             if self.squares[x][start_y].is_some() {
    //                 return false; // a piece is blocking the path
    //             }
    //         }
    //     }
    //
    //     // Destination square must be empty or have an opponent's piece
    //     self.squares[end_x][end_y].is_none() || self.squares[end_x][end_y].map_or(false, |p| p.color != color)
    // }
    fn is_valid_rook_move(&self, start: (usize, usize), end: (usize, usize), color: Color) -> bool {
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        // Rook can only move horizontally or vertically
        if start_x != end_x && start_y != end_y {
            return false; // Not a valid rook move
        }

        // Check if path is clear (no pieces blocking the rook's movement)
        if start_x == end_x {
            let range = if start_y < end_y { start_y + 1..end_y } else { end_y + 1..start_y };
            for y in range {
                if self.squares[start_x][y].is_some() {
                    return false; // Path blocked
                }
            }
        } else {
            let range = if start_x < end_x { start_x + 1..end_x } else { end_x + 1..start_x };
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
    fn is_valid_queen_move(&self, start: (usize, usize), end: (usize, usize), color: Color) -> bool {
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
           self.squares[end_x][end_y].is_none() || self.squares[end_x][end_y].map_or(false, |p| p.color != color)
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
        None
    }

    // evaluate the board (material balance)
    fn evaluate(&self) -> i32 {
        let mut score = 0;

        // iterate over the entire board
        for row in &self.squares {
            for square in row {
                if let Some(piece) = square {
                    let piece_value = match piece.piece_type {
                        PieceType::Pawn => 1,
                        PieceType::Knight => 3,
                        PieceType::Bishop => 3,
                        PieceType::Rook => 5,
                        PieceType::Queen => 9,
                        PieceType::King => 1000,
                    };
                    // add score for white pieces, subtract for black pieces
                    score += if piece.color == Color::White {
                        piece_value
                    } else {
                        -piece_value
                    };
                }
            }
        }
        score
    }

    // make a move on the board
    fn make_move(&mut self, mv: ((usize, usize), (usize, usize))) {
        let (start, end) = mv;
        let (start_x, start_y) = start;
        let (end_x, end_y) = end;

        if let Some(_piece) = self.squares[start_x][start_y].take() {
            // perform the move
            self.squares[end_x][end_y] = self.squares[start_x][start_y].take();
            self.squares[start_x][start_y] = None; // clear the starting position
        }
    }

    // minimax function for decision-making (basic ai)
    fn minimax(Board: &Board, depth: usize, is_maximizing: bool) -> i32 {
        if depth == 0 || Board.is_game_over(Color::White) || Board.is_game_over(Color::Black) {
            return Board.evaluate();
        }

        if is_maximizing {
            let mut max_eval = i32::MIN;  // Initialize to the smallest possible value
            let moves = Board.get_all_moves(Color::White); // white's turn
            for mv in moves {
                let mut Board_copy = Board.clone();
                Board_copy.make_move(mv);
                let eval = Board::minimax(&Board_copy, depth - 1, false); // Recursively call minimax
                max_eval = std::cmp::max(max_eval, eval); // Update max_eval with the maximum value
            }
            max_eval // Return the best evaluation for maximizing player
        } else {
            let mut min_eval = i32::MAX; // Initialize to the largest possible value
            let moves = Board.get_all_moves(Color::Black); // black's turn
            for mv in moves {
                let mut Board_copy = Board.clone();
                Board_copy.make_move(mv);
                let eval = Board::minimax(&Board_copy, depth - 1, true);// Recursively call minimax
                min_eval = std::cmp::min(min_eval, eval); // Update min_eval with the minimum value
            }
            min_eval
        }
    }

    // check if a king is in check (attacked by an opposing piece)
    fn is_in_check(&self, color: Color) -> bool {
        let king_pos = self.find_king(color);
        if let Some((king_x, king_y)) = king_pos {
            let opposing_color = if color == Color::White { Color::Black } else { Color::White };
            for x in 0..8 {
                for y in 0..8 {
                    if let Some(piece) = &self.squares[x][y] {
                        if piece.color == opposing_color {
                            if self.is_valid_move((x, y), (king_x, king_y), opposing_color) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // check if a player has valid moves
    fn has_valid_moves(&self, color: Color) -> bool {
        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = &self.squares[x][y] {
                    if piece.color == color {
                        for dx in 0..8 {
                            for dy in 0..8 {
                                if self.is_valid_move((x, y), (dx, dy), color) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn display_board(Board: &Board) {
        for row in Board.squares.iter() {
            for square in row.iter() {
                match square {
                    Some(piece) => print!("{} ", piece.to_char()),
                    None => print!(". "),
                }
            }
            println!();
        }
    }

    // fn parse_move(move_str: &str) -> Option<(usize, usize, usize, usize)> {
    //     if move_str.len() != 4 {
    //         return None;
    //     }
    //
    //     let start_chars: Vec<char> = move_str[0..2].chars().collect();
    //     let end_chars: Vec<char> = move_str[2..4].chars().collect();
    //
    //     // Parse start position
    //     let start_x = start_chars[0] as usize - 'a' as usize; // 'a' -> 0, 'b' -> 1, etc.
    //     let start_y = start_chars[1] as usize - '1' as usize; // '1' -> 0, '2' -> 1, etc.
    //
    //     // Parse end position
    //     let end_x = end_chars[0] as usize - 'a' as usize;
    //     let end_y = end_chars[1] as usize - '1' as usize;
    //
    //     if start_x < 8 && start_y < 8 && end_x < 8 && end_y < 8 {
    //         Some((start_x, start_y, end_x, end_y))
    //     } else {
    //         None
    //     }
    // }
    fn parse_move(move_str: &str) -> Option<(usize, usize)> {
        if move_str.len() != 2 {
            return None;
        }

        // Parse position
        let x = move_str.chars().nth(0)? as usize - 'a' as usize; // 'a' -> 0, 'b' -> 1, etc.
        let y = move_str.chars().nth(1)? as usize - '1' as usize; // '1' -> 0, '2' -> 1, etc.

        if x < 8 && y < 8 {
            Some((x, y))
        } else {
            None
        }
    }
}

fn main() {
    let mut board = Board::new();
    board.print_board();

    let white_moves = board.get_all_moves(Color::White);
    let black_moves = board.get_all_moves(Color::Black);

    let mut current_player = Color::White; // White starts the game

    println!("White has {} valid moves.", white_moves.len());
    println!("Black has {} valid moves.", black_moves.len());

    println!("Is White in check? {}", board.is_in_check(Color::White));
    println!("Is Black in check? {}", board.is_in_check(Color::Black));

    println!("Is the game over for White? {}", board.is_game_over(Color::White));
    println!("Is the game over for Black? {}", board.is_game_over(Color::Black));

    while !board.is_game_over(current_player) {
        board.print_board();

        // prompt for user input
        //println!("enter your move (e.g., e2e4):");
        println!("player {:?}'s turn", current_player);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        if input.trim().len() != 4 {
            println!("Invalid move format. Use 'e2e4'.");
            continue;
        }

        // Parse start and end positions
        let start = Board::parse_move(&input[0..2]);
        let end = Board::parse_move(&input[2..4]);

        // if let (Some((start_x, start_y, _, _)), Some((end_x, end_y, _, _))) = (start, end) {
        //     if board.is_valid_move((start_x, start_y), (end_x, end_y), current_player) {
        //         board.move_piece((start_x, start_y), (end_x, end_y));
        //         current_player = match current_player {
        //             Color::White => Color::Black,
        //             Color::Black => Color::White,
        //         };
        //     } else {
        //         println!("invalid move, try again.");
        //     }
        //    println!("Game over! Player {:?} has no valid moves.", current_player);
        //    }
        if let (Some((start_x, start_y)), Some((end_x, end_y))) = (start, end) {
            println!("Parsed start: ({}, {}), end: ({}, {})", start_x, start_y, end_x, end_y);
            if board.is_valid_move((start_x, start_y), (end_x, end_y), current_player) {
                board.move_piece((start_x, start_y), (end_x, end_y));
                current_player = match current_player {
                    Color::White => Color::Black,
                    Color::Black => Color::White,
                };
            } else {
                println!("invalid move, try again.");
            }
        } else {
            println!("Game over! Player {:?} has no valid moves.", current_player);
          }
    }
}
