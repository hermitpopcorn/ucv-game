use serde::{
	de::{self, MapAccess},
	ser::SerializeStruct,
	Serialize,
};
use serde_json::json;

use crate::gamemaster::types::{
	Choice, ChoiceOption, GameState, Organizer, Player, Round, RoundState,
};

impl Serialize for Player {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("Player", 3)?;
		state.serialize_field("id", &self.id)?;
		state.serialize_field("name", &self.name)?;
		state.serialize_field("points", &self.points)?;
		state.serialize_field("canVote", &self.can_vote)?;
		state.end()
	}
}

impl<'de> serde::de::Deserialize<'de> for Player {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct PlayerVisitor;

		impl<'de> serde::de::Visitor<'de> for PlayerVisitor {
			type Value = Player;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a Player object")
			}

			fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
			where
				V: MapAccess<'de>,
			{
				let mut id: Option<u8> = None;
				let mut name: Option<String> = None;
				let mut points: Option<usize> = None;
				let mut can_vote: Option<bool> = None;

				while let Some(key) = map.next_key()? {
					match key {
						"id" => {
							id = Some(map.next_value()?);
						}
						"name" => {
							name = Some(map.next_value()?);
						}
						"points" => {
							points = Some(map.next_value()?);
						}
						"canVote" => {
							can_vote = Some(map.next_value()?);
						}
						_ => {
							let _ = map.next_value()?;
						}
					}
				}

				if id.is_none() {
					return Err(de::Error::missing_field("id"));
				}
				if name.is_none() {
					name = Some("".to_owned());
				}

				Ok(Player {
					id: id.unwrap(),
					name: name.unwrap(),
					points,
					can_vote,
				})
			}
		}

		deserializer.deserialize_map(PlayerVisitor)
	}
}

impl Into<serde_json::Value> for Organizer {
	fn into(self) -> serde_json::Value {
		json!({
			"name": self.name,
		})
	}
}

impl Serialize for RoundState {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(self.as_str())
	}
}

impl Serialize for Round {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("Round", 7)?;
		state.serialize_field("id", &self.id)?;
		state.serialize_field("number", &self.number)?;
		state.serialize_field("phase", &self.phase)?;
		state.serialize_field("state", &self.state)?;
		state.serialize_field("question", &self.question)?;
		state.serialize_field("choiceA", &self.choice_a)?;
		state.serialize_field("choiceB", &self.choice_b)?;
		state.end()
	}
}

impl Serialize for ChoiceOption {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let option = match self {
			ChoiceOption::ChoiceA => 'a',
			ChoiceOption::ChoiceB => 'b',
		};

		serializer.serialize_char(option)
	}
}

impl Serialize for Choice {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("Choice", 3)?;
		state.serialize_field("id", &self.id)?;
		state.serialize_field("option", &self.option)?;
		state.serialize_field("lie", &self.lie)?;
		state.end()
	}
}

impl Serialize for GameState {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("GameState", 3)?;
		state.serialize_field("round", &self.round)?;
		state.serialize_field("players", &self.players)?;
		state.serialize_field("choices", &self.choices)?;
		state.end()
	}
}

impl<'de> serde::de::Deserialize<'de> for RoundState {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct RoundStateVisitor;

		impl<'de> serde::de::Visitor<'de> for RoundStateVisitor {
			type Value = RoundState;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a RoundState string")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				match value {
					"standby" => Ok(RoundState::Standby),
					"show-question" => Ok(RoundState::ShowQuestion),
					"show-choices" => Ok(RoundState::ShowChoices),
					"voting-time" => Ok(RoundState::VotingTime),
					"voting-locked" => Ok(RoundState::VotingLocked),
					"show-votes" => Ok(RoundState::ShowVotes),
					"defense" => Ok(RoundState::Defense),
					"show-results" => Ok(RoundState::ShowResults),
					_ => Err(E::custom("invalid RoundState string")),
				}
			}
		}

		deserializer.deserialize_str(RoundStateVisitor)
	}
}

impl<'de> serde::de::Deserialize<'de> for ChoiceOption {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct ChoiceOptionVisitor;

		impl<'de> serde::de::Visitor<'de> for ChoiceOptionVisitor {
			type Value = ChoiceOption;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a ChoiceOption string")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				match value {
					"a" => Ok(ChoiceOption::ChoiceA),
					"b" => Ok(ChoiceOption::ChoiceB),
					_ => Err(E::custom("invalid ChoiceOption string")),
				}
			}
		}

		deserializer.deserialize_str(ChoiceOptionVisitor)
	}
}
