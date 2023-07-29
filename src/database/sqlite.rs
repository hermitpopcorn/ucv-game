use std::{collections::HashMap, fmt::Error};

use anyhow::{bail, Result};
use rusqlite::{
	params,
	types::{FromSql, FromSqlError},
	Connection, OptionalExtension, ToSql,
};

use crate::gamemaster::types::{Choice, ChoiceOption, ChoicesMap, Player, Round, RoundState};

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
				points: Some(row.get(2)?),
				can_vote: Some(row.get(3)?),
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

	fn get_active_round(&self) -> Result<Round> {
		let mut statement = self
			.connection
			.prepare("SELECT id FROM Rounds ORDER BY number DESC, phase DESC LIMIT 1")?;

		let find = statement.query_row(params![], |row| {
			let id: u8 = row.get(0)?;
			Ok(id)
		});

		match find {
			Ok(round_id) => self.get_round_by_id(round_id),
			Err(_) => bail!("Could not find active round"),
		}
	}

	fn create_round(
		&self,
		number: u8,
		phase: u8,
		state: RoundState,
		question: String,
		choice_a: String,
		choice_b: String,
	) -> Result<Round> {
		let mut statement = self.connection.prepare(
			"INSERT INTO Round (number, phase, state, question, choice_a, choice_b)
				VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
		)?;
		let affected =
			statement.execute(params![number, phase, state, question, choice_a, choice_b])?;

		if affected != 1 {
			bail!("Incorrect number of affected rows")
		}

		let last_inserted_id = self.connection.last_insert_rowid();

		self.get_round_by_id(u8::try_from(last_inserted_id)?)
	}

	fn get_choices_by_round_id(&self, round_id: u8) -> Result<ChoicesMap> {
		let mut choices = HashMap::new();

		let mut statement = self.connection.prepare(
			"SELECT player_id, id, option, lie
				FROM Choices WHERE round_id = ?1",
		)?;

		let mut query = statement.query(params![round_id])?;
		while let Some(row) = query.next()? {
			choices.insert(
				row.get(0)?,
				Choice {
					id: row.get(1)?,
					option: row.get(2)?,
					lie: row.get(3)?,
				},
			);
		}

		Ok(choices)
	}
}

impl SqliteDatabase {
	fn get_round_by_id(&self, id: u8) -> Result<Round> {
		let mut statement = self.connection.prepare(
			"SELECT id, number, phase, state, question, choice_a, choice_b
				FROM Rounds WHERE id = ?1",
		)?;

		let find = statement.query_row(params![id], |row| {
			Ok(Round {
				id: row.get(0)?,
				number: row.get(1)?,
				phase: row.get(2)?,
				state: row.get(3)?,
				question: row.get(4)?,
				choice_a: row.get(5)?,
				choice_b: row.get(6)?,
			})
		});

		match find {
			Ok(round) => Ok(round),
			Err(_) => bail!("Could not find active round"),
		}
	}
}

impl ToSql for RoundState {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		match self {
			RoundState::Standby => Ok(0.into()),
			RoundState::ShowQuestion => Ok(1.into()),
			RoundState::ShowChoices => Ok(2.into()),
			RoundState::VotingTime => Ok(3.into()),
			RoundState::VotingLocked => Ok(4.into()),
			RoundState::ShowVotes => Ok(5.into()),
			RoundState::Defense => Ok(6.into()),
			RoundState::ShowResults => Ok(7.into()),
		}
	}
}

impl FromSql for RoundState {
	fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
		match value.as_i64()? {
			0 => Ok(RoundState::Standby),
			1 => Ok(RoundState::ShowQuestion),
			2 => Ok(RoundState::ShowChoices),
			3 => Ok(RoundState::VotingTime),
			4 => Ok(RoundState::VotingLocked),
			5 => Ok(RoundState::ShowVotes),
			6 => Ok(RoundState::Defense),
			7 => Ok(RoundState::ShowResults),
			_ => Err(FromSqlError::Other(Box::new(Error))),
		}
	}
}

impl FromSql for ChoiceOption {
	fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
		match value.as_str()? {
			"a" => Ok(ChoiceOption::ChoiceA),
			"b" => Ok(ChoiceOption::ChoiceB),
			_ => Err(FromSqlError::Other(Box::new(Error))),
		}
	}
}
