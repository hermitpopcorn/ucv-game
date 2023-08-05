use std::{collections::HashMap, net::SocketAddr};

use crossbeam::channel::Sender;

use crate::postmaster::types::InternalMessage;

#[derive(Debug, Clone)]
pub enum ClientStatus {
	Unregistered,
	Registered,
}

#[derive(Debug, Clone)]
pub struct Client {
	pub individual_channel_sender: Sender<InternalMessage>,
	pub status: ClientStatus,
	pub player: Option<Player>,
	pub organizer: Option<Organizer>,
}

pub type ClientsMap = HashMap<SocketAddr, Client>;

#[derive(Debug, Clone)]
pub struct Player {
	pub id: u8,
	pub name: String,
	pub points: Option<usize>,
	pub can_vote: Option<bool>,
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
	pub fn as_str(&self) -> &'static str {
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

#[derive(Debug, Clone)]
pub struct Round {
	pub id: u8,
	pub number: u8,
	pub phase: u8,
	pub state: RoundState,
	pub question: String,
	pub choice_a: String,
	pub choice_b: String,
}

#[derive(Debug, Clone)]
pub enum ChoiceOption {
	ChoiceA,
	ChoiceB,
}

impl ChoiceOption {
	fn as_str(&self) -> &'static str {
		match self {
			ChoiceOption::ChoiceA => "a",
			ChoiceOption::ChoiceB => "b",
		}
	}
}

#[derive(Debug, Clone)]
pub struct Choice {
	pub id: u8,
	pub option: ChoiceOption,
	pub lie: bool,
}

#[derive(Debug, Clone)]
pub struct MarkChoiceLie {
	pub id: u8,
	pub lie: bool,
}

pub type ChoicesMap = HashMap<u8, Choice>;

#[derive(Debug, Clone)]
pub struct GameState {
	pub round: Option<Round>,
	pub players: Vec<Player>,
	pub choices: ChoicesMap,
}
