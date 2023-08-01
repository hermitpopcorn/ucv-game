use serde::{ser::SerializeStruct, Serialize};
use serde_derive::Deserialize;
use serde_json::json;

use crate::gamemaster::types::{
	Choice, ChoiceOption, GameState, Organizer, Player, Round, RoundState,
};

use super::types::{ResponseIdentifier, WebSocketMessage, WebSocketMessageAction};

impl Serialize for Player {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("Player", 3)?;
		state.serialize_field("id", &self.id)?;
		state.serialize_field("name", &self.name)?;
		state.serialize_field("points", &self.points)?;
		state.end()
	}
}

impl Into<serde_json::Value> for Organizer {
	fn into(self) -> serde_json::Value {
		json!({
			"name": self.name,
		})
	}
}

impl Serialize for RoundState {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(self.as_str())
	}
}

impl Serialize for Round {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("Round", 7)?;
		state.serialize_field("id", &self.id)?;
		state.serialize_field("number", &self.number)?;
		state.serialize_field("phase", &self.phase)?;
		state.serialize_field("state", &self.state)?;
		state.serialize_field("question", &self.question)?;
		state.serialize_field("choiceA", &self.choice_a)?;
		state.serialize_field("choiceB", &self.choice_b)?;
		state.end()
	}
}

impl Serialize for ChoiceOption {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let option = match self {
			ChoiceOption::ChoiceA => 'a',
			ChoiceOption::ChoiceB => 'b',
		};

		serializer.serialize_char(option)
	}
}

impl Serialize for Choice {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("Choice", 3)?;
		state.serialize_field("id", &self.id)?;
		state.serialize_field("option", &self.option)?;
		state.serialize_field("lie", &self.lie)?;
		state.end()
	}
}

impl Serialize for GameState {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("GameState", 3)?;
		state.serialize_field("round", &self.round)?;
		state.serialize_field("players", &self.players)?;
		state.serialize_field("choices", &self.choices)?;
		state.end()
	}
}

struct RoundStateVisitor;

impl<'de> serde::de::Visitor<'de> for RoundStateVisitor {
	type Value = RoundState;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		formatter.write_str("a RoundState string")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		match value {
			"standby" => Ok(RoundState::Standby),
			"show-question" => Ok(RoundState::ShowQuestion),
			"show-choices" => Ok(RoundState::ShowChoices),
			"voting-time" => Ok(RoundState::VotingTime),
			"voting-locked" => Ok(RoundState::VotingLocked),
			"show-votes" => Ok(RoundState::ShowVotes),
			"defense" => Ok(RoundState::Defense),
			"show-results" => Ok(RoundState::ShowResults),
			_ => Err(E::custom("invalid RoundState string")),
		}
	}
}

impl<'de> serde::de::Deserialize<'de> for RoundState {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_str(RoundStateVisitor)
	}
}

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
		_ => return None,
	}
}

pub fn make_json_player_identity_response(
	response_id: ResponseIdentifier,
	player: Player,
) -> serde_json::Value {
	json!({
		"responseId": response_id,
		"action": "set-player",
		"payload": player,
	})
}

pub fn make_json_organizer_identity_response(
	response_id: ResponseIdentifier,
	organizer: Organizer,
) -> serde_json::Value {
	let organizer: serde_json::Value = organizer.into();
	json!({
		"responseId": response_id,
		"action": "set-organizer",
		"payload": organizer,
	})
}

pub fn make_json_active_players(
	response_id: ResponseIdentifier,
	active_players: Vec<Player>,
) -> serde_json::Value {
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

pub fn make_json_game_state(
	response_id: ResponseIdentifier,
	game_state: GameState,
) -> serde_json::Value {
	json!({
		"responseId": response_id,
		"action": "set-game-state",
		"payload": game_state,
	})
}

pub fn make_json_round(response_id: ResponseIdentifier, round: Round) -> serde_json::Value {
	json!({
		"responseId": response_id,
		"action": "set-round",
		"payload": round,
	})
}
