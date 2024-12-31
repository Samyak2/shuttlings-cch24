use std::{fmt::Display, sync::RwLock};

use salvo::{
    oapi::{extract::PathParam, BasicType, Content, Object, Schema},
    prelude::*,
};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    Empty,
    Cookie,
    Milk,
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Empty => "⬛",
                Self::Cookie => "🍪",
                Self::Milk => "🥛",
            }
        )
    }
}

impl Default for Item {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, Copy)]
enum GameState {
    NotEnded,
    CookieWon,
    MilkWon,
    NoWinner,
}

impl GameState {
    pub fn is_game_over(&self) -> bool {
        match self {
            GameState::NotEnded => false,
            GameState::CookieWon | GameState::MilkWon | GameState::NoWinner => true,
        }
    }
}

impl From<Item> for GameState {
    fn from(value: Item) -> Self {
        match value {
            Item::Empty => Self::NotEnded,
            Item::Cookie => Self::CookieWon,
            Item::Milk => Self::MilkWon,
        }
    }
}

#[derive(Debug, Clone)]
struct Board {
    board: [[Item; 4]; 4],
    state: GameState,
}

impl Board {
    const fn new() -> Self {
        Self {
            board: [
                [Item::Empty, Item::Empty, Item::Empty, Item::Empty],
                [Item::Empty, Item::Empty, Item::Empty, Item::Empty],
                [Item::Empty, Item::Empty, Item::Empty, Item::Empty],
                [Item::Empty, Item::Empty, Item::Empty, Item::Empty],
            ],
            state: GameState::NotEnded,
        }
    }

    fn check_win(&mut self) {
        // horizontal win
        for row in self.board {
            let first = row[0];
            let all_same = row.iter().all(|v| *v == first);
            if all_same {
                self.state = first.into();
                if self.state.is_game_over() {
                    return;
                }
            }
        }

        // vertical win
        for col_index in 0..4 {
            let first = self.board[0][col_index];
            let all_same = self
                .board
                .iter()
                .map(|row| row[col_index])
                .all(|v| v == first);
            if all_same {
                self.state = first.into();
                if self.state.is_game_over() {
                    return;
                }
            }
        }

        // main diagonal win
        let first = self.board[0][0];
        let all_same = self
            .board
            .iter()
            .enumerate()
            .skip(1)
            .all(|(row_index, row)| row[row_index] == first);
        if all_same {
            self.state = first.into();
            if self.state.is_game_over() {
                return;
            }
        }

        let first = self.board[0][3];
        let all_same = self
            .board
            .iter()
            .enumerate()
            .skip(1)
            .all(|(row_index, row)| row[3 - row_index] == first);
        if all_same {
            self.state = first.into();
            if self.state.is_game_over() {
                return;
            }
        }

        let all_filled = self
            .board
            .iter()
            .all(|row| row.iter().all(|value| *value != Item::Empty));
        if all_filled {
            self.state = GameState::NoWinner;
        }
    }

    pub fn place(&mut self, team: Team, column: usize) -> Result<(), PlaceError> {
        if column > 3 {
            return Err(PlaceError::ColumnNotFound);
        }

        if self.state.is_game_over() {
            return Err(PlaceError::GameOver(self.clone()));
        }

        let row_index = self
            .board
            .iter()
            .enumerate()
            .filter_map(|(index, row)| match row[column] {
                Item::Empty => Some(index),
                _ => None,
            })
            .last();

        match row_index {
            Some(row_index) => {
                self.board[row_index][column] = team.into();
                self.check_win();
                Ok(())
            }
            None => Err(PlaceError::ColumnFull(self.clone())),
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const WALL: &str = "⬜";

        let out = self
            .board
            .map(|row| {
                format!(
                    "{WALL}{}{WALL}",
                    row.map(|state| state.to_string()).join(""),
                )
            })
            .join("\n");
        let out = format!("{}\n{}\n", out, WALL.repeat(6));
        let out = match self.state {
            GameState::NotEnded => out,
            GameState::CookieWon => format!("{}{}\n", out, "🍪 wins!"),
            GameState::MilkWon => format!("{}{}\n", out, "🥛 wins!"),
            GameState::NoWinner => format!("{}{}\n", out, "No winner."),
        };
        write!(f, "{}", out)
    }
}

#[async_trait]
impl Writer for Board {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.render(Text::Plain(self.to_string()));
    }
}

impl EndpointOutRegister for Board {
    fn register(_components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        operation.responses.insert(
            StatusCode::OK.as_str(),
            salvo::oapi::Response::new("success").add_content(
                "text/plain",
                Content::new(Schema::Object(Object::new().schema_type(BasicType::String))),
            ),
        );
    }
}

static BOARD: RwLock<Board> = RwLock::new(Board::new());

#[endpoint(status_codes(200, 500))]
async fn board_route() -> Result<Board, StatusCode> {
    Ok(BOARD
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .clone())
}

#[endpoint(status_codes(200, 500))]
async fn reset_route() -> Result<Board, StatusCode> {
    let mut board = BOARD
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    *board = Board::new();
    Ok(board.clone())
}

#[derive(Debug, Clone, Copy, ToSchema, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Team {
    Milk,
    Cookie,
}

impl From<Team> for Item {
    fn from(value: Team) -> Self {
        match value {
            Team::Milk => Self::Milk,
            Team::Cookie => Self::Cookie,
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum PlaceError {
    #[error("the requested column does not exist")]
    ColumnNotFound,

    #[error("{0}")]
    ColumnFull(Board),

    #[error("{0}")]
    GameOver(Board),

    #[error("internal error")]
    InternalError,
}

#[async_trait]
impl Writer for PlaceError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        match self {
            Self::ColumnNotFound => {
                res.status_code(StatusCode::BAD_REQUEST);
            }
            Self::ColumnFull(_) | Self::GameOver(_) => {
                res.status_code(StatusCode::SERVICE_UNAVAILABLE);
            }
            Self::InternalError => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        res.render(Text::Plain(self.to_string()));
    }
}

impl EndpointOutRegister for PlaceError {
    fn register(_components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        operation.responses.insert(
            StatusCode::BAD_REQUEST.as_str(),
            salvo::oapi::Response::new("bad request").add_content(
                "text/plain",
                Content::new(Schema::Object(Object::new().schema_type(BasicType::String))),
            ),
        );
        operation.responses.insert(
            StatusCode::SERVICE_UNAVAILABLE.as_str(),
            salvo::oapi::Response::new("invalid state").add_content(
                "text/plain",
                Content::new(Schema::Object(Object::new().schema_type(BasicType::String))),
            ),
        );
        operation.responses.insert(
            StatusCode::INTERNAL_SERVER_ERROR.as_str(),
            salvo::oapi::Response::new("internal error").add_content(
                "text/plain",
                Content::new(Schema::Object(Object::new().schema_type(BasicType::String))),
            ),
        );
    }
}

#[endpoint]
async fn place_route(team: PathParam<Team>, column: PathParam<usize>) -> Result<Board, PlaceError> {
    if *column == 0 {
        return Err(PlaceError::ColumnNotFound);
    }

    let mut board = BOARD.write().map_err(|_| PlaceError::InternalError)?;
    board.place(*team, *column - 1)?;
    Ok(board.clone())
}

pub fn get_router() -> Router {
    Router::new()
        .push(Router::with_path("/12/board").get(board_route))
        .push(Router::with_path("/12/reset").post(reset_route))
        .push(Router::with_path("/12/place/<team>/<column>").post(place_route))
}
