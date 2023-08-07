use std::{collections::HashMap, net::SocketAddr, sync::MutexGuard};

use anyhow::{bail, Result};
use crossbeam::channel::Sender;
use log::{debug, warn};

use crate::{
	database::database::{Database, DatabaseAccess},
	postmaster::types::{InternalMessage, InternalMessageAction},
};

use super::types::{Choice, ChoicesMap, Client, ClientsMap, GameState, Player, Round};

pub(super) fn acquire_database_lock(
	database_access: &DatabaseAccess,
) -> Result<MutexGuard<'_, (dyn Database + 'static)>> {
	let db_lock = database_access.try_lock();
	if db_lock.is_err() {
		bail!("Could not get access to database");
	}

	Ok(db_lock.unwrap())
}

pub(super) fn get_individual_channel_sender<'map_lifetime>(
	clients_map: &'map_lifetime ClientsMap,
	address: &SocketAddr,
) -> Result<&'map_lifetime Sender<InternalMessage>> {
	debug!("===== Get ICS");

	let client = clients_map.get(address);
	if client.is_none() {
		bail!("Could not get client from map with address {}", address);
	}

	let client = client.unwrap();
	Ok(&client.individual_channel_sender)
}

pub(super) fn get_players(clients_map: &ClientsMap) -> Vec<(&SocketAddr, &Player)> {
	debug!("===== Get Players");

	let mut existing_ids = vec![];
	let mut players = vec![];
	for (address, client) in clients_map.into_iter() {
		if client.player.is_none() {
			continue;
		}

		let player = client.player.as_ref().unwrap();
		if existing_ids.contains(&player.id) {
			continue;
		}

		players.push((address, player));
		existing_ids.push(player.id);
	}

	players
}

pub(super) fn get_cloned_vector_of_players(clients_map: &ClientsMap) -> Vec<Player> {
	debug!("===== Get cloned list of Players");

	let players = get_players(clients_map);
	players.iter().map(|p| p.1.clone()).collect()
}

pub(super) fn is_player(clients: &ClientsMap, address: &SocketAddr) -> bool {
	debug!("===== Is player?");

	let client: Option<&Client> = clients.get(address);
	if client.is_none() {
		return false;
	}
	let client = client.unwrap();

	return client.player.is_some();
}

pub(super) fn get_organizers(clients_map: &ClientsMap) -> Vec<(&SocketAddr, &Client)> {
	debug!("===== Get Organizers");

	let mut organizers = vec![];

	for (address, client) in clients_map.into_iter() {
		if client.organizer.is_none() {
			continue;
		}

		organizers.push((address, client));
	}

	organizers
}

pub(super) fn is_organizer(clients: &ClientsMap, address: &SocketAddr) -> bool {
	debug!("===== Is organizer?");

	let client = clients.get(address);
	if client.is_none() {
		return false;
	}
	let client = client.unwrap();

	return client.organizer.is_some();
}

pub(super) fn get_active_round(database: &DatabaseAccess) -> Result<Round> {
	debug!("===== Get active round");

	// Acquire lock
	let db_access = acquire_database_lock(database)?;
	let active_round = db_access.get_active_round()?;

	if active_round.is_none() {
		bail!("There is no active round");
	}

	Ok(active_round.unwrap())
}

pub(super) fn compile_choices(database: &DatabaseAccess, round: &Round) -> Result<ChoicesMap> {
	debug!("===== Compile choices");

	let mut choices: ChoicesMap = HashMap::new();

	// Acquire lock
	let db_access = acquire_database_lock(database)?;
	match db_access.get_choices_by_round_id(round.id) {
		Ok(choices_for_active_round) => {
			choices = choices_for_active_round;
		}
		Err(_) => {}
	}

	Ok(choices)
}

pub(super) fn compile_game_state(
	database: &DatabaseAccess,
	clients: &ClientsMap,
) -> Result<GameState> {
	debug!("===== Compile game state");

	let players = get_cloned_vector_of_players(clients);

	let mut choices: ChoicesMap = HashMap::new();

	let round = {
		let db_access = acquire_database_lock(database)?;
		db_access.get_active_round()?
	};

	if round.is_some() {
		choices = compile_choices(database, round.as_ref().unwrap())?;
	}

	Ok(GameState {
		round,
		players,
		choices,
	})
}

pub(super) fn announce_active_players(clients: &ClientsMap) {
	debug!("===== Announce active players");

	let players = get_cloned_vector_of_players(&clients);

	for (address, client) in clients {
		let ics = &client.individual_channel_sender;
		let send = ics.send(InternalMessage {
			payload: InternalMessageAction::ResponseActivePlayers(players.clone()),
			..Default::default()
		});
		if send.is_err() {
			warn!("Could not announce list of active players to: {}", address);
		}
	}
}

pub(super) fn announce_updated_choices(clients: &ClientsMap, choices: ChoicesMap) {
	debug!("===== Announce updated choices");

	for (address, client) in clients {
		let ics = &client.individual_channel_sender;
		let send = ics.send(InternalMessage {
			payload: InternalMessageAction::ResponseUpdatedChoices(choices.clone()),
			..Default::default()
		});
		if send.is_err() {
			warn!("Could not announce updated choices to: {}", address);
		}
	}
}

pub(super) fn announce_choice_to_organizers(clients: &ClientsMap, player: Player, choice: Choice) {
	debug!("===== Announce choice to organizers");

	let organizers = get_organizers(&clients);

	for (address, client) in organizers {
		let ics = &client.individual_channel_sender;
		let send = ics.send(InternalMessage {
			payload: InternalMessageAction::ResponsePlayerChoice(player.clone(), choice.clone()),
			..Default::default()
		});
		if send.is_err() {
			warn!("Could not announce choice to organizer: {}", address);
		}
	}
}

pub(super) fn announce_updated_player(clients: &ClientsMap, player: Player) {
	debug!("===== Announce updated players");

	for (address, client) in clients {
		let ics = &client.individual_channel_sender;
		let send = ics.send(InternalMessage {
			payload: InternalMessageAction::ResponseUpdatedPlayer(player.clone()),
			..Default::default()
		});
		if send.is_err() {
			warn!("Could not announce updated player to: {}", address);
		}
	}
}

pub(super) fn announce_round(
	database: &DatabaseAccess,
	clients: &ClientsMap,
	round: Option<Round>,
) {
	debug!("===== Announce round");

	let announce_round;
	if round.is_none() {
		let active_round = get_active_round(database);
		if active_round.is_err() {
			warn!("Could not get active round: {}", active_round.unwrap_err());
			return;
		}
		announce_round = active_round.unwrap();
	} else {
		announce_round = round.unwrap();
	}

	for (address, client) in clients {
		let ics = &client.individual_channel_sender;
		let send = ics.send(InternalMessage {
			payload: InternalMessageAction::ResponseRound(announce_round.clone()),
			..Default::default()
		});
		if send.is_err() {
			warn!("Could not announce round state to: {}", address);
		}
	}
}

pub(super) fn allow_all_active_players_to_vote(
	database: &DatabaseAccess,
	clients: &ClientsMap,
) -> Result<()> {
	debug!("===== Allow all active players to vote");

	let db_access = acquire_database_lock(database)?;

	for (_, client) in clients {
		if client.player.is_none() {
			continue;
		}

		let player = client.player.as_ref().unwrap();
		let updated_player = db_access.mark_player(player.id, None, Some(true))?;
		announce_updated_player(clients, updated_player);
	}

	Ok(())
}
