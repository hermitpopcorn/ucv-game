use serde_derive::Deserialize;

use crate::{
	gamemaster::types::{ChoiceOption, MarkChoiceLie, Player, Round, RoundState},
	postmaster::types::{ResponseIdentifier, WebSocketMessage, WebSocketMessageAction},
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonAction {
	response_id: ResponseIdentifier,
	action: String,
}

#[derive(Deserialize, Debug)]
struct JsonMessagePayload {
	payload: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct JsonRound {
	id: u8,
	number: u8,
	phase: u8,
	state: RoundState,
	question: String,
	choiceA: String,
	choiceB: String,
}

impl JsonRound {
	fn as_round(self) -> Round {
		Round {
			id: self.id,
			number: self.number,
			phase: self.phase,
			state: self.state,
			question: self.question,
			choice_a: self.choiceA,
			choice_b: self.choiceB,
		}
	}
}

#[derive(Deserialize, Debug)]
struct JsonSetRoundPayload {
	payload: JsonRound,
}

#[derive(Deserialize, Debug)]
struct JsonSetChoiceOptionPayload {
	payload: ChoiceOption,
}

#[derive(Deserialize, Debug)]
struct JsonSetPlayerPayload {
	payload: Player,
}

#[derive(Deserialize, Debug)]
struct JsonMarkChoicePayload {
	payload: MarkChoiceLie,
}

#[derive(Deserialize, Debug)]
struct JsonSetPlayerPointsPayload {
	payload: Player,
}

pub fn parse_message(message: String) -> Option<WebSocketMessage> {
	let parse = serde_json::from_str(&message);
	if parse.is_err() {
		return None;
	}

	let json: JsonAction = parse.unwrap();
	match json.action.as_str() {
		"login-player" => {
			let parsed_payload: Result<JsonMessagePayload, _> = serde_json::from_str(&message);
			if parsed_payload.is_err() {
				return None;
			}
			let parsed_payload = parsed_payload.unwrap();

			return Some(WebSocketMessage {
				response_id: json.response_id,
				action: WebSocketMessageAction::LoginPlayer(parsed_payload.payload.unwrap()),
			});
		}
		"login-organizer" => {
			let parsed_payload: Result<JsonMessagePayload, _> = serde_json::from_str(&message);
			if parsed_payload.is_err() {
				return None;
			}
			let parsed_payload = parsed_payload.unwrap();

			return Some(WebSocketMessage {
				response_id: json.response_id,
				action: WebSocketMessageAction::LoginOrganizer(parsed_payload.payload.unwrap()),
			});
		}
		"get-game-state" => {
			return Some(WebSocketMessage {
				response_id: json.response_id,
				action: WebSocketMessageAction::RetrieveGameState(),
			});
		}
		"set-round" => {
			let parsed_payload: Result<JsonSetRoundPayload, _> = serde_json::from_str(&message);
			if parsed_payload.is_err() {
				return None;
			}
			let parsed_payload = parsed_payload.unwrap();

			return Some(WebSocketMessage {
				response_id: json.response_id,
				action: WebSocketMessageAction::SetRound(parsed_payload.payload.as_round()),
			});
		}
		"set-vote-is-lie" => {
			let parsed_payload: Result<JsonMarkChoicePayload, _> = serde_json::from_str(&message);
			if parsed_payload.is_err() {
				return None;
			}
			let parsed_payload = parsed_payload.unwrap();

			return Some(WebSocketMessage {
				response_id: json.response_id,
				action: WebSocketMessageAction::MarkChoiceLie(parsed_payload.payload),
			});
		}
		"set-choice" => {
			let parsed_payload: Result<JsonSetChoiceOptionPayload, _> =
				serde_json::from_str(&message);
			if parsed_payload.is_err() {
				return None;
			}
			let parsed_payload = parsed_payload.unwrap();

			return Some(WebSocketMessage {
				response_id: json.response_id,
				action: WebSocketMessageAction::SetChoiceOption(parsed_payload.payload),
			});
		}
		"set-player-can-vote" => {
			let parsed_payload: Result<JsonSetPlayerPayload, _> = serde_json::from_str(&message);
			if parsed_payload.is_err() {
				return None;
			}
			let parsed_payload = parsed_payload.unwrap();

			return Some(WebSocketMessage {
				response_id: json.response_id,
				action: WebSocketMessageAction::SetPlayer(parsed_payload.payload),
			});
		}
		"set-player-points" => {
			let parsed_payload: Result<JsonSetPlayerPointsPayload, _> =
				serde_json::from_str(&message);
			if parsed_payload.is_err() {
				return None;
			}
			let parsed_payload = parsed_payload.unwrap();

			return Some(WebSocketMessage {
				response_id: json.response_id,
				action: WebSocketMessageAction::SetPlayer(parsed_payload.payload),
			});
		}
		_ => return None,
	}
}
