use serde_json::json;

use crate::{
	gamemaster::types::{Choice, ChoicesMap, GameState, Organizer, Player, PlayerMap, Round},
	postmaster::types::ResponseIdentifier,
};

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
	active_players: PlayerMap,
) -> serde_json::Value {
	json!({
		"responseId": response_id,
		"action": "refresh-active-players-list",
		"payload": active_players,
	})
}

pub fn make_json_updated_player(
	response_id: ResponseIdentifier,
	updated_player: Player,
) -> serde_json::Value {
	json!({
		"responseId": response_id,
		"action": "update-player",
		"payload": updated_player,
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

pub fn make_json_updated_choices(
	response_id: ResponseIdentifier,
	choices_map: ChoicesMap,
) -> serde_json::Value {
	json!({
		"responseId": response_id,
		"action": "set-choices",
		"payload": choices_map,
	})
}

pub fn make_json_player_choice(
	response_id: ResponseIdentifier,
	player: Player,
	choice: Choice,
) -> serde_json::Value {
	json!({
		"responseId": response_id,
		"action": "set-player-choice",
		"payload": {
			"player": player,
			"choice": choice,
		},
	})
}
