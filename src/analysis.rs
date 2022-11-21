use std::cmp::Ordering;

use serde::{ser::SerializeStruct, Deserialize, Serialize};

type Board = [[Option<Player>; 3]; 3];

fn get_empty_squares(board: &Board) -> Vec<(usize, usize)> {
    let mut empty_squares = Vec::<(usize, usize)>::new();

    for (r, column) in board.iter().enumerate() {
        for (c, square) in column.iter().enumerate() {
            if square.is_none() {
                empty_squares.push((c, r));
            }
        }
    }

    empty_squares
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GameStatus {
    Won,
    Drew,
    Lost,
    Ongoing,
}

fn check_side(square1: Player, square2: Player) -> GameStatus {
    if square1 == square2 {
        GameStatus::Won
    } else {
        GameStatus::Lost
    }
}

fn check_status(board: &Board, player: Player) -> GameStatus {
    let b = board.iter().copied().flatten().collect::<Vec<_>>();

    // A1 - C3 diagonal check
    if b[0].is_some() && b[4] == b[0] && b[8] == b[0] {
        return check_side(player, b[0].unwrap());
    }

    // A3 - C1 diagonal check
    if b[6].is_some() && b[4] == b[6] && b[2] == b[6] {
        return check_side(player, b[6].unwrap());
    }

    // Horizontal & vertical check
    for i in 0..3usize {
        let r = i * 3;

        // horizontal
        if b[r].is_some() && b[r + 1] == b[r] && b[r + 2] == b[r] {
            return check_side(player, b[i * 3].unwrap());
        }

        // vertical
        if b[i].is_some() && b[i + 3] == b[i] && b[i + 6] == b[i] {
            return check_side(player, b[i].unwrap());
        }
    }

    if b.iter().any(|s| s.is_none()) {
        GameStatus::Ongoing
    } else {
        GameStatus::Drew
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    X,
    O,
}

impl Player {
    fn switch(&self) -> Player {
        match *self {
            Self::O => Self::X,
            Self::X => Self::O,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Info {
    turn: Player,
    board: Board,
}

impl Info {
    pub fn analyze(&self) -> AnalysisResult {
        let mut analysis = AnalysisResult::new();

        let status = check_status(&self.board, self.turn);

        match status {
            GameStatus::Lost => {
                analysis.eval = Some(if self.turn == Player::X {
                    Evaluation::OWins
                } else {
                    Evaluation::XWins
                });
            }
            GameStatus::Drew => analysis.eval = Some(Evaluation::Draw),
            GameStatus::Won => {
                analysis.eval = Some(if self.turn == Player::X {
                    Evaluation::XWins
                } else {
                    Evaluation::OWins
                });
            }
            GameStatus::Ongoing => {}
        };

        if status != GameStatus::Ongoing {
            return analysis;
        }

        let empty_squares = get_empty_squares(&self.board);

        for (c, r) in empty_squares {
            let mut board_copy = self.board;
            board_copy[r][c] = Some(self.turn);
            let status = check_status(&board_copy, self.turn);

            match status {
                GameStatus::Won => analysis.moves.push(Move::winning_move((c, r))),
                GameStatus::Drew => analysis.moves.push(Move::draw_move((c, r))),
                GameStatus::Lost => analysis.moves.push(Move::losing_move((c, r))),
                GameStatus::Ongoing => {
                    let next_move = Info {
                        turn: self.turn.switch(),
                        board: board_copy,
                    };

                    let new_analysis = next_move.analyze();

                    let mut win_count = 0.0;
                    let mut draw_count = 0.0;
                    let mut lose_count = 0.0;

                    let analysis_len = new_analysis.moves.len() as f32;

                    for m in new_analysis.moves.iter() {
                        match m.analysis {
                            Analysis::Lose => lose_count += 1.0,
                            Analysis::Draw => draw_count += 1.0,
                            Analysis::Win => win_count += 1.0,
                        }
                    }

                    let new_analysis = new_analysis
                        .moves
                        .iter()
                        .reduce(|prev, cur| {
                            if cur.analysis > prev.analysis {
                                cur
                            } else {
                                prev
                            }
                        })
                        .unwrap()
                        .analysis
                        .flip();

                    analysis.moves.push(Move {
                        square: (c, r),
                        analysis: new_analysis,
                        chances: Chances {
                            win: (lose_count / analysis_len * 100.0).round(),
                            draw: (draw_count / analysis_len * 100.0).round(),
                            lose: (win_count / analysis_len * 100.0).round(),
                        },
                    });
                }
            }
        }

        let best_move = analysis
            .moves
            .iter()
            .reduce(|prev, cur| {
                if cur.analysis > prev.analysis {
                    cur
                } else {
                    prev
                }
            })
            .unwrap();

        let eval = match best_move.analysis {
            Analysis::Lose => {
                if self.turn == Player::X {
                    Evaluation::OWins
                } else {
                    Evaluation::XWins
                }
            }
            Analysis::Draw => Evaluation::Draw,
            Analysis::Win => {
                if self.turn == Player::X {
                    Evaluation::XWins
                } else {
                    Evaluation::OWins
                }
            }
        };

        analysis.eval = Some(eval);
        analysis.moves.sort_unstable_by(|m1, m2| {
            let cmp = m1.analysis.cmp(&m2.analysis);
            if cmp != Ordering::Equal {
                return cmp;
            }

            let cmp_win = m1.chances.win.total_cmp(&m2.chances.win);
            let cmp_draw = m1.chances.draw.total_cmp(&m2.chances.draw);
            let cmp_lose = m1.chances.lose.total_cmp(&m2.chances.lose);

            if cmp_win != Ordering::Equal {
                cmp_win
            } else if cmp_draw != Ordering::Equal {
                cmp_draw
            } else {
                cmp_lose
            }
        });
        analysis.moves.reverse();

        analysis
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
pub enum Evaluation {
    XWins,
    OWins,
    Draw,
}

#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum Analysis {
    Lose,
    Draw,
    Win,
}

impl Analysis {
    pub fn flip(&self) -> Analysis {
        match *self {
            Self::Win => Self::Lose,
            Self::Draw => Self::Draw,
            Self::Lose => Self::Win,
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
pub struct Chances {
    pub win: f32,
    pub draw: f32,
    pub lose: f32,
}

impl Chances {
    pub const WIN: Chances = Chances {
        win: 100.0,
        draw: 0.0,
        lose: 0.0,
    };

    pub const DRAW: Chances = Chances {
        win: 0.0,
        draw: 100.0,
        lose: 0.0,
    };

    pub const LOSE: Chances = Chances {
        win: 0.0,
        draw: 0.0,
        lose: 100.0,
    };
}

fn get_square_annotation(square: (usize, usize)) -> String {
    let letter = ["A", "B", "C"];
    format!("{}{}", letter[square.0], square.1 + 1)
}

#[derive(Debug)]
pub struct Move {
    pub square: (usize, usize),
    pub analysis: Analysis,
    pub chances: Chances,
}

impl Move {
    pub const fn winning_move(square: (usize, usize)) -> Self {
        Move {
            square,
            analysis: Analysis::Win,
            chances: Chances::WIN,
        }
    }

    pub const fn draw_move(square: (usize, usize)) -> Self {
        Move {
            square,
            analysis: Analysis::Draw,
            chances: Chances::DRAW,
        }
    }

    pub const fn losing_move(square: (usize, usize)) -> Self {
        Move {
            square,
            analysis: Analysis::Lose,
            chances: Chances::LOSE,
        }
    }
}

impl Serialize for Move {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Move", 2)?;

        state.serialize_field("square", &get_square_annotation(self.square))?;
        state.serialize_field("analysis", &self.analysis)?;
        state.serialize_field("chances", &self.chances)?;

        state.end()
    }
}

#[derive(Serialize, Debug)]
pub struct AnalysisResult {
    pub moves: Vec<Move>,
    pub eval: Option<Evaluation>,
}

impl AnalysisResult {
    pub fn new() -> Self {
        Self {
            moves: Vec::new(),
            eval: None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::analysis::GameStatus;

    use super::{check_status, Analysis, Info, Player};

    #[test]
    fn check_wins_test() {
        assert_eq!(
            check_status(
                &[
                    [Some(Player::X), Some(Player::X), Some(Player::X)],
                    [None, None, None],
                    [None, None, None],
                ],
                Player::X,
            ),
            GameStatus::Won
        );

        assert_eq!(
            check_status(
                &[
                    [None, Some(Player::O), None],
                    [None, Some(Player::O), None],
                    [None, Some(Player::O), None],
                ],
                Player::O,
            ),
            GameStatus::Won
        );

        assert_eq!(
            check_status(
                &[
                    [Some(Player::X), Some(Player::X), Some(Player::X)],
                    [None, None, None],
                    [None, None, None],
                ],
                Player::O,
            ),
            GameStatus::Lost
        );

        assert_eq!(
            check_status(
                &[
                    [None, Some(Player::O), None],
                    [None, Some(Player::O), None],
                    [None, Some(Player::O), None],
                ],
                Player::X,
            ),
            GameStatus::Lost
        );

        assert_eq!(
            check_status(
                &[
                    [Some(Player::X), None, None],
                    [None, Some(Player::X), None],
                    [None, None, Some(Player::X)],
                ],
                Player::O
            ),
            GameStatus::Lost
        );

        assert_eq!(
            check_status(
                &[
                    [None, None, Some(Player::O)],
                    [None, Some(Player::O), None],
                    [Some(Player::O), None, None],
                ],
                Player::O
            ),
            GameStatus::Won
        );

        // Ongoing
        assert_eq!(
            check_status(
                &[
                    [None, None, None],
                    [None, Some(Player::X), None],
                    [Some(Player::O), None, None],
                ],
                Player::X
            ),
            GameStatus::Ongoing,
        );

        assert_eq!(
            check_status(
                &[
                    [None, Some(Player::O), Some(Player::X)],
                    [Some(Player::O), Some(Player::X), Some(Player::X)],
                    [Some(Player::O), Some(Player::X), Some(Player::O)],
                ],
                Player::X
            ),
            GameStatus::Ongoing,
        );

        assert_eq!(
            check_status(
                &[
                    [Some(Player::X), Some(Player::O), Some(Player::X)],
                    [Some(Player::O), Some(Player::X), Some(Player::X)],
                    [Some(Player::O), Some(Player::X), Some(Player::O)],
                ],
                Player::X
            ),
            GameStatus::Drew,
        );
    }

    #[test]
    fn analysis_comparison_test() {
        assert!(Analysis::Win > Analysis::Draw);
        assert!(Analysis::Draw > Analysis::Lose);
        assert!(Analysis::Lose < Analysis::Draw);
        assert!(Analysis::Draw < Analysis::Win);
        assert!(Analysis::Draw >= Analysis::Draw);
        assert!(Analysis::Win >= Analysis::Win);
        assert!(Analysis::Lose <= Analysis::Lose);
        assert!(Analysis::Win >= Analysis::Win);
    }

    #[test]
    fn analyze_test() {
        let info = Info {
            turn: Player::O,
            board: [
                [Some(Player::X), Some(Player::O), Some(Player::X)],
                [Some(Player::X), Some(Player::X), Some(Player::O)],
                [None, None, Some(Player::O)],
            ],
        };

        println!("{:?}", info.analyze());

        let info = Info {
            turn: Player::O,
            board: [
                [Some(Player::X), Some(Player::O), Some(Player::X)],
                [Some(Player::X), Some(Player::X), Some(Player::O)],
                [Some(Player::X), None, Some(Player::O)],
            ],
        };

        println!("{:?}", info.analyze());

        let info = Info {
            turn: Player::X,
            board: [
                [Some(Player::X), Some(Player::O), Some(Player::O)],
                [Some(Player::O), Some(Player::X), None],
                [None, Some(Player::X), Some(Player::O)],
            ],
        };

        println!("{:?}", info.analyze());
    }
}
