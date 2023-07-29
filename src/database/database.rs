use anyhow::Result;

use crate::types::Player;

pub trait Database: Send {
	fn initialize_database(&self) -> Result<()>;

	fn find_player(&self, name: &str) -> Result<Player>;
	fn create_player(&self, name: &str) -> Result<Player>;
	fn find_or_create_player(&self, name: &str) -> Result<Player>;
}
