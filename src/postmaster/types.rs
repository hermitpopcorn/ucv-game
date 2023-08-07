use std::net::SocketAddr;

use crossbeam::channel::Sender;

use crate::gamemaster::types::{
	Choice, ChoiceOption, ChoicesMap, GameState, MarkChoiceLie, Organizer, Player, Round,
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
	RequestMarkChoiceLie(SocketAddr, MarkChoiceLie),
	RequestSetChoiceOption(SocketAddr, ChoiceOption),

	// From Organizer client to GM
	RequestRegisterOrganizer(SocketAddr, String),
	RequestSetRound(SocketAddr, Round),
	RequestSetPlayerData(SocketAddr, Player),
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
