use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::gamemaster::types::{Choice, ChoiceOption, ChoicesMap, Player, Round, RoundState};

pub type DatabaseAccess = Arc<Mutex<dyn Database>>;

pub trait Database: Send {
	fn initialize_database(&self) -> Result<()>;

	fn find_player_by_id(&self, id: u8) -> Result<Option<Player>>;
	fn find_player_by_name(&self, name: &str) -> Result<Option<Player>>;
	fn create_player(&self, name: &str) -> Result<Player>;
	fn find_or_create_player(&self, name: &str) -> Result<Player>;
	fn mark_player(&self, id: u8, points: Option<usize>, can_vote: Option<bool>) -> Result<Player>;

	fn get_active_round(&self) -> Result<Option<Round>>;
	fn find_round_by_number_and_phase(&self, number: u8, phase: u8) -> Result<Option<Round>>;
	fn create_round(
		&self,
		number: u8,
		phase: u8,
		state: RoundState,
		question: String,
		choice_a: String,
		choice_b: String,
	) -> Result<Round>;
	fn update_round(
		&self,
		number: u8,
		phase: u8,
		state: Option<RoundState>,
		question: Option<String>,
		choice_a: Option<String>,
		choice_b: Option<String>,
	) -> Result<Round>;

	fn find_choice_by_round_and_player(
		&self,
		round_id: u8,
		player_id: u8,
	) -> Result<Option<Choice>>;
	fn update_or_create_choice(
		&self,
		round_id: u8,
		player_id: u8,
		choice: ChoiceOption,
	) -> Result<Choice>;
	fn mark_choice(&self, choice_id: u8, lie: Option<bool>) -> Result<()>;

	fn get_choices_by_round_id(&self, round_id: u8) -> Result<ChoicesMap>;

	fn check_player_is_allowed_to_vote(&self, player_id: u8) -> Result<bool>;
}
