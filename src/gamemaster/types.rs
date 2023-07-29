use std::{collections::HashMap, net::SocketAddr};

use crossbeam::channel::Sender;

use crate::postmaster::types::InternalMessage;

#[derive(Debug, Clone)]
pub enum ClientType {
	Blank,
	Player,
	Organizer,
}

#[derive(Debug, Clone)]
pub struct Client {
	pub individual_channel_sender: Sender<InternalMessage>,
	pub client_type: ClientType,
	pub player: Option<Player>,
	pub organizer: Option<Organizer>,
}

pub type ClientsMap = HashMap<SocketAddr, Client>;

#[derive(Debug, Clone)]
pub struct Player {
	pub id: u8,
	pub name: String,
	pub points: usize,
	pub can_vote: bool,
}

#[derive(Debug, Clone)]
pub struct Organizer {
	pub name: String,
}

#[derive(Debug, Clone)]
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
