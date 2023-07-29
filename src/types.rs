use std::{
	collections::HashMap,
	net::SocketAddr,
	sync::{Arc, Mutex},
};

use crossbeam::channel::Sender;
use serde_json::json;

use crate::database::database::Database;

pub type DatabaseAccess = Arc<Mutex<dyn Database>>;

#[derive(Debug, Clone)]
pub struct Client {
	pub individual_channel_sender: Sender<ChannelMessage>,
	pub player: Option<Player>,
}

pub type ClientsMap = HashMap<SocketAddr, Client>;

pub type ResponseIdentifier = Option<String>;

#[derive(Debug, Clone)]
pub enum ChannelMessageAction {
	// Responses by GM
	ResponseOkay,
	ResponseNotOkay(String),
	ResponseIdentity(Player),
	ResponseActivePlayers(Vec<Player>),

	// From Client to GM
	ExitClient(SocketAddr),
	RegisterClient(SocketAddr, Sender<ChannelMessage>),
	RegisterActivePlayer(SocketAddr, ResponseIdentifier, String),
	RetrieveActivePlayers(SocketAddr),
}

#[derive(Debug, Clone)]
pub struct ChannelMessage {
	pub payload: ChannelMessageAction,
	pub response_id: ResponseIdentifier,
}

impl Default for ChannelMessage {
	fn default() -> ChannelMessage {
		ChannelMessage {
			payload: ChannelMessageAction::ResponseOkay,
			response_id: None,
		}
	}
}

#[derive(Debug, Clone)]
pub enum WebSocketMessageAction {
	Login(String),
}

#[derive(Debug, Clone)]
pub struct WebSocketMessage {
	pub response_id: ResponseIdentifier,
	pub action: WebSocketMessageAction,
}

#[derive(Debug, Clone)]
pub struct Player {
	pub id: u8,
	pub name: String,
	pub points: usize,
	pub can_vote: bool,
}

impl Into<serde_json::Value> for Player {
	fn into(self) -> serde_json::Value {
		json!({
			"id": self.id,
			"name": self.name,
			"points": self.points,
		})
	}
}

pub enum RoundState {
	Standby,
	ShowQuestion,
	ShowChoices,
	VotingTime,
	VotingLocked,
	ShowVotes,
	Defense,
	ShowResults,
}

impl RoundState {
	fn as_str(&self) -> &'static str {
		match self {
			RoundState::Standby => "standby",
			RoundState::ShowQuestion => "show-question",
			RoundState::ShowChoices => "show-choices",
			RoundState::VotingTime => "voting-time",
			RoundState::VotingLocked => "voting-locked",
			RoundState::ShowVotes => "show-votes",
			RoundState::Defense => "defense",
			RoundState::ShowResults => "show-results",
		}
	}
}
