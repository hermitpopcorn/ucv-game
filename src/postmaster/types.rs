use std::net::SocketAddr;

use crossbeam::channel::Sender;

use crate::gamemaster::types::{GameState, Organizer, Player, Round};

pub type ResponseIdentifier = Option<String>;

#[derive(Debug, Clone)]
pub enum InternalMessageAction {
	// Responses by GM
	ResponseOkay,
	ResponseNotOkay(String),
	ResponsePlayerIdentity(Player),
	ResponseActivePlayers(Vec<Player>),
	ResponseGameState(GameState),
	ResponseRound(Round),

	ResponseOrganizerIdentity(Organizer),

	// From Client to GM
	ExitClient(SocketAddr),
	RegisterClient(SocketAddr, Sender<InternalMessage>),
	RegisterActivePlayer(SocketAddr, String),
	RetrieveActivePlayers(SocketAddr),
	RetrieveGameState(SocketAddr),

	RegisterOrganizer(SocketAddr, String),
	SetRound(SocketAddr, Round),
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
}

#[derive(Debug, Clone)]
pub struct WebSocketMessage {
	pub response_id: ResponseIdentifier,
	pub action: WebSocketMessageAction,
}
