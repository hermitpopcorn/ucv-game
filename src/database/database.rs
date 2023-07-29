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
	fn create_round(
		&self,
		number: u8,
		phase: u8,
		state: RoundState,
		question: String,
		choice_a: String,
		choice_b: String,
	) -> Result<Round>;

	fn get_choices_by_round_id(&self, round_id: u8) -> Result<ChoicesMap>;
}
