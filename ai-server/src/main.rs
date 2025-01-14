#[macro_use] extern crate rocket;

use std::collections::HashSet;

use game::{max_best_evaluation, min_best_evaluation};
use itertools::Itertools;
use rocket::fairing::{Fairing, Info, Kind};

use rocket::http::Header;
use rocket::{Request, Response};
use rocket::response::status::BadRequest;
use serde_json::{json, Value};

use crate::board::{Board, Position};
use crate::bot::Bot;
use crate::game::{Action, Game, Player};

mod board;
mod errors;
mod game;
mod bot;


fn serialize_result(game: &Game) -> Value {
    let mut json = json!({
        "board": game.board().to_string(),
    });

    if game.is_over() {
        json["winner"] = serde_json::to_value(game.winner().map(|p| p.to_string()))
            .unwrap_or_else(|_| Value::Null);
    }
    
    json
}


#[get("/")]
fn index() -> &'static str {
    "Hello World!"
}

#[get("/initial-board")]
fn initial_board() -> String {
    Board::new().to_string()
}

#[get("/evaluate?<board>")]
fn evaluate(board: String) -> Result<String, BadRequest<String>> {
    let board = Board::parse(board);
    if board.is_err() {
        return Err(BadRequest(Some("Invalid board".to_string())));
    }

    let evaluation = Game::parse(board.unwrap(), Player::default()).evaluate();

    let range = max_best_evaluation() - min_best_evaluation();
    let normalized = (evaluation - min_best_evaluation()) as f32 / range as f32;
    Ok(normalized.to_string())
}

#[get("/result?<board>&<position>&<player>")]
fn result(board: String, position: String, player: String) -> Result<String, BadRequest<String>> {
    let board = Board::parse(board);
    if board.is_err() {
        return Err(BadRequest(Some("Invalid board".to_string())));
    }
    
    let player = player.chars().next();
    if player.is_none() {
        return Err(BadRequest(Some("Invalid player".to_string())));
    }
    
    let player = Player::parse(player.unwrap());
    if player.is_err() {
        return Err(BadRequest(Some("Invalid player".to_string())));
    }
    let player = player.unwrap();

    let game = Game::parse(board.unwrap(), player.clone());
    let action = Action::parse(player, Position::parse(position).unwrap());
    
    if !game.actions(player).contains(&action) {
        return Err(BadRequest(Some("Invalid action for the given player".to_string())));
    }

    let game = game.result(&action);
    
    Ok(serialize_result(&game).to_string())
}

#[get("/actions?<board>&<player>")]
fn actions(board: String, player: String) -> Result<String, BadRequest<String>> {
    let board = Board::parse(board);
    if board.is_err() {
        return Err(BadRequest(Some("Invalid board".to_string())));
    }

    let player = player.chars().next();
    if player.is_none() {
        return Err(BadRequest(Some("Invalid player".to_string())));
    }

    let player = Player::parse(player.unwrap());
    if player.is_err() {
        return Err(BadRequest(Some("Invalid player".to_string())));
    }
    let player = player.unwrap();
    
    let game = Game::parse(board.unwrap(), player);
    Ok(Value::Array(
        game.actions(player)
            .map(|a| Value::String(a.to_string()))
            .collect_vec()
    ).to_string())
}

#[get("/decide?<board>&<intelligence>")]
fn decide(board: String, intelligence: u32) -> Result<String, BadRequest<String>> {
    let mut bot = Bot::new(intelligence);
    let board = Board::parse(board);
    if board.is_err() {
        return Err(BadRequest(Some("Invalid board".to_string())));
    }
    
    let game = Game::parse(board.unwrap(), Player::Bot);
    
    let decision = bot.decide(&game);
    
    if decision.is_err() { // No available actions
        let json = json!({
            "decision": Value::Null,
            "result": serialize_result(&game),
        });
        return Ok(json.to_string());
    }
    
    let (action, game) = decision.unwrap();
    
    let json = json!({
        "decision": action.to_string(),
        "result": serialize_result(&game),
    });
    
    Ok(json.to_string())
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let allowed_origins: HashSet<String> = [
        "http://localhost:443",
        "https://localhost",
        "https://localhost:80",  
        "http://localhost:8080",
        "http://localhost",
        "http://desdemona.seoulsky.org",
        "http://desdemona.seoulsky.org:443",
        "https://desdemona.seoulsky.org",
        ].iter()
        .map(|s| s.to_string())
        .collect();

    rocket::build()
        .mount("/api", routes![index, initial_board, evaluate, result, actions, decide])
        .attach(Cors::new(allowed_origins))
        .launch()
        .await?;

    Ok(())
}

pub struct Cors {
    allowed_origins: HashSet<String>,
}

impl Cors {
    pub fn new(allowed_origins: HashSet<String>) -> Cors {
        Cors { allowed_origins }
    }
}

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "CORS Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let origin = request.headers().get_one("Origin").unwrap_or("");

        if self.allowed_origins.contains(origin) {
            response.set_header(Header::new("Access-Control-Allow-Origin", origin));
            response.set_header(Header::new("Access-Control-Allow-Methods", "GET"));
            response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
            response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        }
    }
}
