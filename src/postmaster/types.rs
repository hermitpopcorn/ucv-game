use std::net::SocketAddr;

use crossbeam::channel::Sender;

use crate::gamemaster::types::{
	Choice, ChoiceOption, ChoicesMap, GameState, MarkChoiceLie, Organizer, Player, Round,
};

pub type ResponseIdentifier = Option<String>;

#[derive(Debug, Clone)]
pub enum InternalMessageAction {
	// Responses by GM
	ResponseOkay,
	ResponseNotOkay(String),
	ResponsePlayerIdentity(Player),
	ResponseActivePlayers(Vec<Player>),
	ResponseUpdatedPlayer(Player),
	ResponseGameState(GameState),
	ResponseRound(Round),
	ResponseUpdatedChoices(ChoicesMap),

	ResponseOrganizerIdentity(Organizer),
	ResponsePlayerChoice(Player, Choice),

	// From Client to GM
	ExitClient(SocketAddr),
	RegisterClient(SocketAddr, Sender<InternalMessage>),
	RegisterActivePlayer(SocketAddr, String),
	RetrieveActivePlayers(SocketAddr),
	RetrieveGameState(SocketAddr),
	MarkChoiceLie(SocketAddr, MarkChoiceLie),
	SetChoiceOption(SocketAddr, ChoiceOption),

	RegisterOrganizer(SocketAddr, String),
	SetRound(SocketAddr, Round),
	SetPlayer(SocketAddr, Player),
}

#[derive(Debug, Clone)]
pub struct InternalMessage {
	pub payload: InternalMessageAction,
	pub response_id: ResponseIdentifier,
}

impl Default for InternalMessage {
	fn default() -> InternalMessage {
		InternalMessage {
			payload: InternalMessageAction::ResponseOkay,
			response_id: None,
		}
	}
}

#[derive(Debug, Clone)]
pub enum WebSocketMessageAction {
	LoginPlayer(String),
	LoginOrganizer(String),
	RetrieveGameState(),
	SetRound(Round),
	SetChoiceOption(ChoiceOption),
	SetPlayer(Player),
	MarkChoiceLie(MarkChoiceLie),
}

#[derive(Debug, Clone)]
pub struct WebSocketMessage {
	pub response_id: ResponseIdentifier,
	pub action: WebSocketMessageAction,
}
