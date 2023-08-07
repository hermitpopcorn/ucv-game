use std::{collections::HashMap, net::SocketAddr};

use anyhow::{anyhow, bail, Result};
use crossbeam::channel::{Receiver, Sender};
use log::{debug, error, info};

use crate::{
	database::database::DatabaseAccess,
	gamemaster::{
		helpers::{
			acquire_database_lock, allow_all_active_players_to_vote, announce_active_players,
			announce_choice_to_organizers, announce_round, announce_updated_choices,
			announce_updated_player, compile_choices, compile_game_state, get_active_round,
			get_individual_channel_sender, is_organizer, is_player,
		},
		types::RoundState,
	},
	postmaster::types::{InternalMessage, InternalMessageAction, ResponseIdentifier},
};

use super::types::{ChoiceOption, Client, ClientStatus, ClientsMap, Organizer, Round};

pub async fn start_gamemaster(
	gm_channel_receiver: Receiver<InternalMessage>,
	database: DatabaseAccess,
) -> Result<()> {
	let mut clients: ClientsMap = HashMap::new();

	loop {
		let received_message = gm_channel_receiver.recv();
		if received_message.is_err() {
			error!("Error when trying to receive channel message");
			break;
		}

		let received_message = received_message.unwrap();
		let response = match received_message.payload {
			InternalMessageAction::RequestRegisterClient(address, individual_channel_sender) => {
				let r = process_register_client(&mut clients, address, individual_channel_sender);
				Some(r)
			}
			InternalMessageAction::RequestRegisterActivePlayer(address, name) => {
				let r = process_register_active_player(
					&database,
					&mut clients,
					address,
					received_message.response_id,
					name,
				);
				Some(r)
			}
			InternalMessageAction::RequestRegisterOrganizer(address, password) => {
				let r = process_register_organizer(
					&mut clients,
					address,
					received_message.response_id,
					password,
				);
				Some(r)
			}
			InternalMessageAction::ExitClient(address) => {
				let r = process_exit_client(&mut clients, address);
				Some(r)
			}
			InternalMessageAction::RequestGameState(address) => {
				let r = process_retrieve_game_state(
					&database,
					&clients,
					address,
					received_message.response_id,
				);
				Some(r)
			}
			InternalMessageAction::RequestSetRound(address, round) => {
				let r = process_set_round(
					&database,
					&clients,
					address,
					received_message.response_id,
					round,
				);
				Some(r)
			}
			InternalMessageAction::RequestSetChoiceOption(address, choice) => {
				let r = process_set_choice(
					&database,
					&clients,
					address,
					received_message.response_id,
					choice,
				);
				Some(r)
			}
			InternalMessageAction::RequestMarkPlayer(address, id, points, can_vote) => {
				let r = process_mark_player(
					&database,
					&clients,
					address,
					received_message.response_id,
					id,
					points,
					can_vote,
				);
				Some(r)
			}
			InternalMessageAction::RequestMarkChoice(address, id, lie) => {
				let r = process_mark_choice(
					&database,
					&clients,
					address,
					received_message.response_id,
					id,
					lie,
				);
				Some(r)
			}
			_ => None,
		};

		// Log a warning to output if the processing did not return Ok
		if let Some(response) = response {
			if response.is_err() {
				error!("{}", response.unwrap_err());
			}
		}
	}

	Ok(())
}

fn process_register_client(
	clients: &mut ClientsMap,
	address: SocketAddr,
	individual_channel_sender: Sender<InternalMessage>,
) -> Result<()> {
	debug!("===== Register client");
	clients.insert(
		address.clone(),
		Client {
			individual_channel_sender,
			status: ClientStatus::Unregistered,
			player: None,
			organizer: None,
		},
	);

	debug!("===== Client registered. Sending okay response...");
	let client = clients
		.get(&address)
		.ok_or(anyhow!("Could not get inserted client"))?;
	client.individual_channel_sender.send(InternalMessage {
		payload: InternalMessageAction::ResponseOkay,
		..Default::default()
	})?;

	Ok(())
}

fn process_register_active_player(
	database: &DatabaseAccess,
	clients: &mut ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	name: String,
) -> Result<()> {
	debug!("===== Register player");

	// Limit player name to 12 characters
	let trimmed_name = &name[0..std::cmp::min(name.len(), 12)];

	let player = {
		let db_access = acquire_database_lock(database)?;

		// Find player data in database
		debug!("Finding/creating player ({})...", &trimmed_name);
		db_access.find_or_create_player(&trimmed_name)
	}?;

	// Loop clients, see if player already exists
	for (iter_address, iter_client) in clients.iter() {
		if *iter_address == address {
			continue;
		}
		if iter_client.player.is_none() {
			continue;
		}

		if iter_client.player.as_ref().unwrap().id == player.id {
			// Conflict
			let ics = get_individual_channel_sender(clients, &address)?;
			ics.send(InternalMessage {
				payload: InternalMessageAction::ResponseNotOkay(
					"That player is already connected on a different device.".to_owned(),
				),
				response_id,
				..Default::default()
			})?;
			return Ok(());
		}
	}

	// Insert into clients map
	clients.entry(address).and_modify(|c| {
		c.status = ClientStatus::Registered;
		c.player = Some(player.clone());
	});
	info!("Connected players updated");

	// Send response confirming identity
	let ics = get_individual_channel_sender(&clients, &address)?;
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponsePlayerIdentity(player),
		response_id,
		..Default::default()
	})?;
	debug!("Response (player identity confirmation) sent");

	// Announce to other clients
	announce_active_players(&clients);

	Ok(())
}

fn process_register_organizer(
	clients: &mut ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	password: String,
) -> Result<()> {
	debug!("===== Register organizer");

	// Check if organizer key is valid
	let the_password = "minorityrule"; // TODO do not hardcode this
	if !password.eq(the_password) {
		let ics = get_individual_channel_sender(&clients, &address)?;
		ics.send(InternalMessage {
			payload: InternalMessageAction::ResponseNotOkay(
				"The organizer password is incorrect.".to_owned(),
			),
			response_id,
			..Default::default()
		})?;
		return Ok(());
	}

	// Make up some name and update clients map
	let organizer = Organizer {
		name: random_word::gen(random_word::Lang::En).to_owned(),
	};
	clients.entry(address).and_modify(|c| {
		c.status = ClientStatus::Registered;
		c.organizer = Some(organizer.clone());
	});
	debug!("Marked client as organizer");

	let ics = get_individual_channel_sender(&clients, &address)?;
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseOrganizerIdentity(organizer),
		response_id: response_id,
	})?;
	debug!("Response (organizer identity confirmation) sent");

	Ok(())
}

fn process_exit_client(clients: &mut ClientsMap, address: SocketAddr) -> Result<()> {
	debug!("===== Exit client");

	clients.remove(&address);
	debug!("Removed client: {}", address);

	announce_active_players(clients);

	Ok(())
}

fn process_retrieve_game_state(
	database: &DatabaseAccess,
	clients: &ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
) -> Result<()> {
	debug!("===== Retrieve game state");

	let game_state = compile_game_state(database, clients)?;

	let ics = get_individual_channel_sender(&clients, &address)?;
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseGameState(game_state),
		response_id,
		..Default::default()
	})?;

	Ok(())
}

fn process_set_round(
	database: &DatabaseAccess,
	clients: &ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	round: Round,
) -> Result<()> {
	debug!("===== Set round");

	if !is_organizer(clients, &address) {
		bail!("Set round request came from a non-organizer");
	}

	let is_new_round;

	let round = {
		let db_access = acquire_database_lock(database)?;
		let find_round = db_access.find_round_by_number_and_phase(round.number, round.phase)?;

		let updated_round = match find_round {
			Some(db_round) => {
				is_new_round = false;
				db_access.update_round(
					db_round.number,
					db_round.phase,
					Some(round.state),
					Some(round.question),
					Some(round.choice_a),
					Some(round.choice_b),
				)?
			}
			None => {
				is_new_round = true;
				db_access.create_round(
					round.number,
					round.phase,
					round.state,
					round.question,
					round.choice_a,
					round.choice_b,
				)?
			}
		};

		updated_round
	};

	// Announce updated round
	announce_round(database, clients, Some(round.clone()));

	// Announce choices if the round is new or in a state where choices are being shown
	if is_new_round
		|| round.state == RoundState::ShowVotes
		|| round.state == RoundState::VotingTime
		|| round.state == RoundState::VotingLocked
		|| round.state == RoundState::ShowResults
	{
		let updated_choices = compile_choices(database, &round)?;
		announce_updated_choices(clients, updated_choices);
	}

	// Set can_vote to true to all active players if it's the standby state of a new round
	if round.phase == 1 && round.state == RoundState::Standby {
		allow_all_active_players_to_vote(database, clients)?;
	}

	let ics = get_individual_channel_sender(&clients, &address)?;
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseOkay,
		response_id,
		..Default::default()
	})?;

	Ok(())
}

fn process_set_choice(
	database: &DatabaseAccess,
	clients: &ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	option: ChoiceOption,
) -> Result<()> {
	debug!("===== Set choice");

	if !is_player(clients, &address) {
		bail!("Set choice request came from a non-player");
	}

	let round = {
		let db_access = acquire_database_lock(database)?;
		db_access.get_active_round()?
	};
	if round.is_none() {
		bail!("Trying to set choice when no round is active");
	}
	let round = round.unwrap();

	let client = clients.get(&address);
	if client.is_none() {
		bail!("Could not find the client");
	}
	let player = client.unwrap().clone().player;
	if player.is_none() {
		bail!("The client is not a player");
	}
	let player = player.unwrap();

	let is_player_allowed_to_vote = {
		let db_access = acquire_database_lock(database)?;
		db_access.check_player_is_allowed_to_vote(player.id)?
	};

	if is_player_allowed_to_vote == false {
		let ics = get_individual_channel_sender(&clients, &address)?;
		ics.send(InternalMessage {
			payload: InternalMessageAction::ResponseNotOkay(
				"You are not allowed to vote.".to_owned(),
			),
			response_id,
			..Default::default()
		})?;
		return Ok(());
	}

	let set_choice = {
		let db_access = acquire_database_lock(database)?;
		db_access.update_or_create_choice(round.id, player.id, option)
	}?;

	announce_choice_to_organizers(clients, player, set_choice);

	let ics = get_individual_channel_sender(&clients, &address)?;
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseOkay,
		response_id,
		..Default::default()
	})?;

	Ok(())
}

fn process_mark_choice(
	database: &DatabaseAccess,
	clients: &ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	id: u8,
	lie: Option<bool>,
) -> Result<()> {
	debug!("===== Mark choice");

	if !is_organizer(clients, &address) {
		bail!("Mark choice request came from a non-organizer");
	}

	let round = get_active_round(database)?;
	{
		let db_access = acquire_database_lock(database)?;
		db_access.mark_choice(id, lie)
	}?;

	let updated_choices = compile_choices(database, &round)?;
	announce_updated_choices(clients, updated_choices);

	let ics = get_individual_channel_sender(&clients, &address)?;
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseOkay,
		response_id,
		..Default::default()
	})?;

	Ok(())
}

fn process_mark_player(
	database: &DatabaseAccess,
	clients: &ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	id: u8,
	points: Option<usize>,
	can_vote: Option<bool>,
) -> Result<()> {
	debug!("===== Mark player");

	if !is_organizer(clients, &address) {
		bail!("Mark player request came from a non-organizer");
	}

	let updated_player = {
		let db_access = acquire_database_lock(database)?;
		db_access.mark_player(id, points, can_vote)
	}?;

	announce_updated_player(clients, updated_player);

	let ics = get_individual_channel_sender(&clients, &address)?;
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseOkay,
		response_id,
		..Default::default()
	})?;

	Ok(())
}
