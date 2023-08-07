use std::net::SocketAddr;

use crossbeam::channel::Sender;

use crate::gamemaster::types::{
	Choice, ChoiceOption, ChoicesMap, GameState, Organizer, Player, Round,
};

pub type ResponseIdentifier = Option<String>;

#[derive(Debug, Clone)]
pub enum InternalMessageAction {
	// General responses
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
	RequestRegisterClient(SocketAddr, Sender<InternalMessage>),

	// From Player client to GM
	RequestRegisterActivePlayer(SocketAddr, String),
	RequestGameState(SocketAddr),
	RequestSetChoiceOption(SocketAddr, ChoiceOption),

	// From Organizer client to GM
	RequestRegisterOrganizer(SocketAddr, String),
	RequestSetRound(SocketAddr, Round),
	RequestMarkPlayer(SocketAddr, u8, Option<usize>, Option<bool>),
	RequestMarkChoice(SocketAddr, u8, Option<bool>),
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
	MarkPlayer(u8, Option<usize>, Option<bool>),
	MarkChoice(u8, Option<bool>),
}

#[derive(Debug, Clone)]
pub struct WebSocketMessage {
	pub response_id: ResponseIdentifier,
	pub action: WebSocketMessageAction,
}
