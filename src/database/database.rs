use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::gamemaster::types::{ChoicesMap, Player, Round, RoundState};

pub type DatabaseAccess = Arc<Mutex<dyn Database>>;

pub trait Database: Send {
	fn initialize_database(&self) -> Result<()>;

	fn find_player(&self, name: &str) -> Result<Player>;
	fn create_player(&self, name: &str) -> Result<Player>;
	fn find_or_create_player(&self, name: &str) -> Result<Player>;

	fn get_active_round(&self) -> Result<Round>;
	fn find_round_by_number_and_phase(&self, number: u8, phase: u8) -> Result<Round>;
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

	fn get_choices_by_round_id(&self, round_id: u8) -> Result<ChoicesMap>;
}
