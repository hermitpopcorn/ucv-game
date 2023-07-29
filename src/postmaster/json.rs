use serde_derive::Deserialize;
use serde_json::json;

use crate::gamemaster::types::Player;

use super::types::{ResponseIdentifier, WebSocketMessage, WebSocketMessageAction};

impl Into<serde_json::Value> for Player {
	fn into(self) -> serde_json::Value {
		json!({
			"id": self.id,
			"name": self.name,
			"points": self.points,
		})
	}
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonMessage {
	response_id: ResponseIdentifier,
	action: String,
	payload: String,
}

pub fn parse_message(message: String) -> Option<WebSocketMessage> {
	let parse = serde_json::from_str(&message);
	if parse.is_err() {
		return None;
	}

	let json: JsonMessage = parse.unwrap();
	match json.action.as_str() {
		"login" => {
			return Some(WebSocketMessage {
				response_id: json.response_id,
				action: WebSocketMessageAction::Login(json.payload),
			})
		}
		_ => return None,
	}
}

pub fn make_json_identity_response(
	response_id: ResponseIdentifier,
	player: Player,
) -> serde_json::Value {
	let player: serde_json::Value = player.into();
	json!({
		"responseId": response_id,
		"action": "set-player",
		"payload": player,
	})
}

pub fn make_json_active_players(
	response_id: ResponseIdentifier,
	active_players: Vec<Player>,
) -> serde_json::Value {
	let active_players: serde_json::Value = active_players.into();
	json!({
		"responseId": response_id,
		"action": "refresh-active-players-list",
		"payload": active_players,
	})
}

pub fn make_json_alert_message(
	response_id: ResponseIdentifier,
	message: String,
) -> serde_json::Value {
	json!({
		"responseId": response_id,
		"action": "show-message",
		"payload": message,
	})
}

pub fn make_json_okay_response(response_id: ResponseIdentifier) -> serde_json::Value {
	json!({
		"responseId": response_id,
		"action": "ok",
	})
}

pub fn make_json_not_okay_response(
	response_id: ResponseIdentifier,
	message: String,
) -> serde_json::Value {
	json!({
		"responseId": response_id,
		"action": "ng",
		"payload": message,
	})
}
