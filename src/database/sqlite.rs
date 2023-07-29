use anyhow::{bail, Result};
use rusqlite::{params, Connection, OptionalExtension};

use crate::types::Player;

use super::database::Database;

pub struct SqliteDatabase {
	connection: Connection,
}

impl SqliteDatabase {
	pub fn new(path: &str) -> Self {
		let connection = Connection::open(path).unwrap();

		let new = Self {
			connection: connection,
		};
		new.initialize_database().unwrap();

		new
	}
}

impl Database for SqliteDatabase {
	fn initialize_database(&self) -> Result<()> {
		let mut statement = self
			.connection
			.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'Players'")?;
		let check = statement.query_row([], |_row| Ok(())).optional()?;

		if check.is_none() {
			self.connection.execute(
				"CREATE TABLE 'Players' (
					'id'          INTEGER,
					'name'        VARCHAR(255) NOT NULL,
					'points'      INTEGER DEFAULT 0,
					'can_vote'    BOOLEAN DEFAULT FALSE,
					PRIMARY KEY('id' AUTOINCREMENT)
				)",
				[],
			)?;
			self.connection
				.execute("CREATE UNIQUE INDEX 'name_index' ON 'Players' ('name')", [])?;
		}

		let mut statement = self
			.connection
			.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'Rounds'")?;
		let check = statement.query_row([], |_row| Ok(())).optional()?;

		if check.is_none() {
			self.connection.execute(
				"CREATE TABLE 'Rounds' (
					'id'       INTEGER,
					'number'   INTEGER NOT NULL,
					'phase'    INTEGER NOT NULL,
					'state'    INTEGER DEFAULT 0,
					'question' VARCHAR(255) NOT NULL,
					'choice_a' VARCHAR(255) NOT NULL,
					'choice_b' VARCHAR(255) NOT NULL,
					PRIMARY KEY('id' AUTOINCREMENT)
				)",
				[],
			)?;
			self.connection.execute(
				"CREATE UNIQUE INDEX 'number_phase_index' ON 'Rounds' ('number', 'phase')",
				[],
			)?;
		}

		let mut statement = self
			.connection
			.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'Choices'")?;
		let check = statement.query_row([], |_row| Ok(())).optional()?;

		if check.is_none() {
			self.connection.execute(
				"CREATE TABLE 'Choices' (
					'id'        INTEGER NOT NULL,
					'round_id'  INTEGER NOT NULL,
					'player_id' INTEGER NOT NULL,
					'option'    TEXT(1) NOT NULL,
					'lie'       INTEGER(1) DEFAULT 0 NOT NULL,
					PRIMARY KEY('id' AUTOINCREMENT)
				)",
				[],
			)?;
			self.connection.execute(
				"CREATE INDEX 'round_id_index' ON 'Choices' ('round_id')",
				[],
			)?;
			self.connection.execute(
				"CREATE INDEX 'player_id_index' ON 'Choices' ('player_id')",
				[],
			)?;
		}

		Ok(())
	}

	fn find_player(&self, name: &str) -> Result<Player> {
		let mut statement = self
			.connection
			.prepare("SELECT id, name, points, can_vote FROM Players WHERE name = ?1")?;

		let find = statement.query_row(params![name], |row| {
			Ok(Player {
				id: row.get(0)?,
				name: row.get(1)?,
				points: row.get(2)?,
				can_vote: row.get(3)?,
			})
		});

		match find {
			Ok(player) => Ok(player),
			Err(_) => bail!("Could not find player"),
		}
	}

	fn create_player(&self, name: &str) -> Result<Player> {
		let mut statement = self
			.connection
			.prepare("INSERT INTO Players (name) VALUES (?1)")?;
		let affected = statement.execute(params![name])?;

		if affected != 1 {
			bail!("Incorrect number of affected rows")
		}

		self.find_player(name)
	}

	fn find_or_create_player(&self, name: &str) -> Result<Player> {
		match self.find_player(name) {
			Ok(player) => Ok(player),
			Err(_) => self.create_player(name),
		}
	}
}
