use std::{collections::HashMap, fmt::Error};

use anyhow::{anyhow, bail, Result};
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

	fn find_player_by_id(&self, id: u8) -> Result<Option<Player>> {
		let mut statement = self
			.connection
			.prepare("SELECT id, name, points, can_vote FROM Players WHERE id = ?1")?;

		let find = statement
			.query_row(params![id], |row| {
				Ok(Player {
					id: row.get(0)?,
					name: row.get(1)?,
					points: Some(row.get(2)?),
					can_vote: Some(row.get(3)?),
				})
			})
			.optional()?;

		Ok(find)
	}

	fn find_player_by_name(&self, name: &str) -> Result<Option<Player>> {
		let mut statement = self
			.connection
			.prepare("SELECT id, name, points, can_vote FROM Players WHERE name = ?1")?;

		let find = statement
			.query_row(params![name], |row| {
				Ok(Player {
					id: row.get(0)?,
					name: row.get(1)?,
					points: Some(row.get(2)?),
					can_vote: Some(row.get(3)?),
				})
			})
			.optional()?;

		Ok(find)
	}

	fn create_player(&self, name: &str) -> Result<Player> {
		let mut statement = self
			.connection
			.prepare("INSERT INTO Players (name) VALUES (?1)")?;
		let affected = statement.execute(params![name])?;

		if affected != 1 {
			bail!("Incorrect number of affected rows")
		}

		self.find_player_by_name(name)?
			.ok_or(anyhow!("Could not find crated player"))
	}

	fn find_or_create_player(&self, name: &str) -> Result<Player> {
		match self.find_player_by_name(name)? {
			Some(player) => Ok(player),
			None => self.create_player(name),
		}
	}

	fn update_player(
		&self,
		id: u8,
		name: Option<&str>,
		points: Option<usize>,
		can_vote: Option<bool>,
	) -> Result<Player> {
		let mut columns: Vec<&str> = vec![];

		if name.is_some_and(|name| name.len() > 0) {
			columns.push("name = :name");
		}
		if points.is_some() {
			columns.push("points = :points");
		}
		if can_vote.is_some() {
			columns.push("can_vote = :canvote");
		}

		let columns = columns.join(", ");

		let mut statement = self
			.connection
			.prepare(format!("UPDATE Players SET {} WHERE id = :id", columns).as_str())?;

		let get_index = statement.parameter_index(":name")?;
		if let Some(name_index) = get_index {
			statement.raw_bind_parameter(name_index, name.unwrap())?;
		}
		let get_index = statement.parameter_index(":points")?;
		if let Some(points_index) = get_index {
			statement.raw_bind_parameter(points_index, points.unwrap())?;
		}
		let get_index = statement.parameter_index(":canvote")?;
		if let Some(can_vote_index) = get_index {
			statement.raw_bind_parameter(can_vote_index, can_vote.unwrap())?;
		}

		let get_index = statement.parameter_index(":id")?;
		statement.raw_bind_parameter(get_index.unwrap(), id)?;
		let update = statement.raw_execute()?;

		if update != 1 {
			bail!("Could not update the player");
		}
		Ok(self.find_player_by_id(id)?.unwrap())
	}

	fn get_active_round(&self) -> Result<Option<Round>> {
		let mut statement = self
			.connection
			.prepare("SELECT id FROM Rounds ORDER BY number DESC, phase DESC LIMIT 1")?;

		let find = statement
			.query_row(params![], |row| {
				let id: u8 = row.get(0)?;
				Ok(id)
			})
			.optional()?;

		match find {
			Some(round_id) => self.get_round_by_id(round_id),
			None => Ok(None),
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
			"INSERT INTO Rounds (number, phase, state, question, choice_a, choice_b)
				VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
		)?;
		let affected =
			statement.execute(params![number, phase, state, question, choice_a, choice_b])?;

		if affected != 1 {
			bail!("Incorrect number of affected rows")
		}

		let last_inserted_id = self.connection.last_insert_rowid();

		self.get_round_by_id(u8::try_from(last_inserted_id)?)?
			.ok_or(anyhow!("Could not find created round"))
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

	fn find_round_by_number_and_phase(&self, number: u8, phase: u8) -> Result<Option<Round>> {
		let mut statement = self.connection.prepare(
			"SELECT id, number, phase, state, question, choice_a, choice_b
			FROM Rounds WHERE number = ?1 AND phase = ?2",
		)?;

		let find = statement
			.query_row(params![number, phase], |row| {
				Ok(Round {
					id: row.get(0)?,
					number: row.get(1)?,
					phase: row.get(2)?,
					state: row.get(3)?,
					question: row.get(4)?,
					choice_a: row.get(5)?,
					choice_b: row.get(6)?,
				})
			})
			.optional()?;

		Ok(find)
	}

	fn update_round(
		&self,
		number: u8,
		phase: u8,
		state: Option<RoundState>,
		question: Option<String>,
		choice_a: Option<String>,
		choice_b: Option<String>,
	) -> Result<Round> {
		let round = self.find_round_by_number_and_phase(number, phase)?;
		if round.is_none() {
			bail!("Could not find round");
		}
		let mut round = round.unwrap();

		if let Some(new_state) = state {
			round.state = new_state;
		}
		if let Some(new_question) = question {
			round.question = new_question;
		}
		if let Some(new_choice_a) = choice_a {
			round.choice_a = new_choice_a;
		}
		if let Some(new_choice_b) = choice_b {
			round.choice_b = new_choice_b;
		}

		let mut statement = self.connection.prepare(
			"UPDATE Rounds SET state = ?1, question = ?2, choice_a = ?3, choice_b = ?4
				WHERE number = ?5 AND phase = ?6",
		)?;
		statement.execute(params![
			round.state,
			round.question,
			round.choice_a,
			round.choice_b,
			round.number,
			round.phase
		])?;

		Ok(round)
	}

	fn mark_choice_lie(&self, choice_id: u8, lie: bool) -> Result<()> {
		let mut statement = self
			.connection
			.prepare("UPDATE Choices SET lie = ?1 WHERE id = ?2")?;
		let update = statement.execute(params![lie, choice_id])?;

		if update != 1 {
			bail!("Could not mark choice");
		}
		Ok(())
	}

	fn find_choice_by_round_and_player(
		&self,
		round_id: u8,
		player_id: u8,
	) -> Result<Option<Choice>> {
		let mut statement = self.connection.prepare(
			"SELECT id, option, lie FROM Choices WHERE round_id = ?1 AND player_id = ?2",
		)?;

		let find = statement
			.query_row(params![round_id, player_id], |row| {
				Ok(Choice {
					id: row.get(0)?,
					option: row.get(1)?,
					lie: row.get(2)?,
				})
			})
			.optional()?;

		Ok(find)
	}

	fn update_or_create_choice(
		&self,
		round_id: u8,
		player_id: u8,
		option: ChoiceOption,
	) -> Result<Choice> {
		let find = self.find_choice_by_round_and_player(round_id, player_id)?;

		let sql = match find {
			Some(choice) => {
				let mut statement = self
					.connection
					.prepare("UPDATE Choices SET option = ?1 WHERE id = ?2")?;
				statement.execute(params![option, choice.id])
			}
			None => {
				let mut statement = self.connection.prepare(
					"INSERT INTO Choices (round_id, player_id, option) VALUES (?1, ?2, ?3)",
				)?;
				statement.execute(params![round_id, player_id, option])
			}
		};

		if sql.is_err() {
			bail!("Could not insert/update choice");
		}

		let refind = self
			.find_choice_by_round_and_player(round_id, player_id)?
			.ok_or(anyhow!("Could not refind the inserted/updated choice"))?;

		Ok(Choice {
			id: refind.id,
			option: refind.option,
			lie: refind.lie,
		})
	}

	fn check_player_is_allowed_to_vote(&self, player_id: u8) -> Result<bool> {
		let mut statement = self
			.connection
			.prepare("SELECT can_vote FROM Players WHERE id = ?1")?;

		let find = statement.query_row(params![player_id], |row| {
			let can_vote: bool = row.get(0)?;
			Ok(can_vote)
		});
		if find.is_err() {
			bail!("Could not find player");
		}

		Ok(find.unwrap())
	}
}

impl SqliteDatabase {
	fn get_round_by_id(&self, id: u8) -> Result<Option<Round>> {
		let mut statement = self.connection.prepare(
			"SELECT id, number, phase, state, question, choice_a, choice_b
				FROM Rounds WHERE id = ?1",
		)?;

		let find = statement
			.query_row(params![id], |row| {
				Ok(Round {
					id: row.get(0)?,
					number: row.get(1)?,
					phase: row.get(2)?,
					state: row.get(3)?,
					question: row.get(4)?,
					choice_a: row.get(5)?,
					choice_b: row.get(6)?,
				})
			})
			.optional()?;

		Ok(find)
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

impl ToSql for ChoiceOption {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		match self {
			ChoiceOption::ChoiceA => Ok("a".into()),
			ChoiceOption::ChoiceB => Ok("b".into()),
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
