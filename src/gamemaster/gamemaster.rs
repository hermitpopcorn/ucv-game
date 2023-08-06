use std::{collections::HashMap, net::SocketAddr};

use anyhow::{bail, Result};
use crossbeam::channel::{Receiver, Sender};
use log::{debug, error, info, warn};

use crate::{
	database::database::DatabaseAccess,
	gamemaster::types::RoundState,
	postmaster::types::{InternalMessage, InternalMessageAction, ResponseIdentifier},
};

use super::types::{
	Choice, ChoiceOption, ChoicesMap, Client, ClientStatus, ClientsMap, GameState, MarkChoiceLie,
	Organizer, Player, Round,
};

pub async fn start_gamemaster(
	gm_channel_receiver: Receiver<InternalMessage>,
	database: DatabaseAccess,
) -> Result<()> {
	let mut clients: ClientsMap = HashMap::new();

	loop {
		let received_message = gm_channel_receiver.recv();
		if received_message.is_err() {
			warn!("Error when trying to receive channel message");
			break;
		}

		let received_message = received_message.unwrap();
		match received_message.payload {
			InternalMessageAction::RegisterClient(address, individual_channel_sender) => {
				register_client(&mut clients, address, individual_channel_sender);
			}
			InternalMessageAction::RegisterActivePlayer(address, name) => {
				register_active_player(
					&database,
					&mut clients,
					address,
					received_message.response_id,
					name,
				);
			}
			InternalMessageAction::RegisterOrganizer(address, password) => {
				register_organizer(
					&mut clients,
					address,
					received_message.response_id,
					password,
				);
			}
			InternalMessageAction::RetrieveActivePlayers(address) => {
				retrieve_active_players(&clients, address);
			}
			InternalMessageAction::ExitClient(address) => {
				exit_client(&mut clients, address);
			}
			InternalMessageAction::RetrieveGameState(address) => {
				retrieve_game_state(&database, &clients, address, received_message.response_id);
			}
			InternalMessageAction::SetRound(address, round) => {
				set_round(
					&database,
					&clients,
					address,
					received_message.response_id,
					round,
				);
			}
			InternalMessageAction::SetChoiceOption(address, choice) => {
				set_choice(
					&database,
					&clients,
					address,
					received_message.response_id,
					choice,
				);
			}
			InternalMessageAction::SetPlayer(address, player) => {
				set_player(
					&database,
					&clients,
					address,
					received_message.response_id,
					player,
				);
			}
			InternalMessageAction::MarkChoiceLie(address, mark) => {
				mark_choice_lie(
					&database,
					&clients,
					address,
					received_message.response_id,
					mark,
				);
			}
			_ => continue,
		}
	}

	Ok(())
}

fn get_individual_channel_sender<'map_lifetime>(
	clients_map: &'map_lifetime ClientsMap,
	address: &SocketAddr,
) -> &'map_lifetime Sender<InternalMessage> {
	debug!("===== Get ICS");

	let client = clients_map
		.get(address)
		.expect("Could not find the client when searching for individual channel");
	&client.individual_channel_sender
}

fn get_players(clients_map: &ClientsMap) -> Vec<Player> {
	debug!("===== Get Players");

	let mut existing_ids = vec![];
	let mut players = vec![];
	for (_, client) in clients_map.into_iter() {
		if client.player.is_none() {
			continue;
		}

		let player = client.player.as_ref().unwrap();
		if existing_ids.contains(&player.id) {
			continue;
		}

		players.push(player.clone());
		existing_ids.push(player.id);
	}

	players
}

fn get_organizers(clients_map: &ClientsMap) -> Vec<(&SocketAddr, &Client)> {
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

fn register_client(
	clients: &mut ClientsMap,
	address: SocketAddr,
	individual_channel_sender: Sender<InternalMessage>,
) {
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
		.expect("Could not get inserted client");
	client
		.individual_channel_sender
		.send(InternalMessage {
			payload: InternalMessageAction::ResponseOkay,
			..Default::default()
		})
		.expect("Could not send client registration okay response");
}

fn register_active_player(
	database: &DatabaseAccess,
	clients: &mut ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	name: String,
) {
	debug!("===== Register player");

	// Limit player name to 12 characters
	let trimmed_name = &name[0..std::cmp::min(name.len(), 12)];

	// Get lock to database
	let db_access = database
		.try_lock()
		.expect("Could not get access to database");

	// Find player data in database
	debug!("Finding/creating player ({})...", &trimmed_name);
	let player = db_access
		.find_or_create_player(&trimmed_name)
		.expect(&format!(
			"Could not find or create player: {}.",
			&trimmed_name
		));

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
			clients
				.get(&address)
				.expect("Could not find current client in clients map")
				.individual_channel_sender
				.send(InternalMessage {
					payload: InternalMessageAction::ResponseNotOkay(
						"That player is already connected on a different device.".to_owned(),
					),
					response_id,
					..Default::default()
				})
				.expect("Could not send NG response");
			return;
		}
	}

	clients.entry(address).and_modify(|c| {
		c.status = ClientStatus::Registered;
		c.player = Some(player.clone());
	});
	info!("Connected players updated");

	let ics = get_individual_channel_sender(&clients, &address);

	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponsePlayerIdentity(player),
		response_id,
		..Default::default()
	})
	.expect("Could not send identity confirmation response");
	debug!("Response (player identity confirmation) sent");

	announce_active_players(&clients);
}

fn announce_active_players(clients: &ClientsMap) {
	debug!("===== Announce active players");

	let players = get_players(&clients);

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

fn announce_updated_choices(clients: &ClientsMap, choices: ChoicesMap) {
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

fn announce_updated_player(clients: &ClientsMap, player: Player) {
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

fn register_organizer(
	clients: &mut ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	password: String,
) {
	debug!("===== Register organizer");

	// Check if organizer key is valid
	let the_password = "minorityrule"; // TODO do not hardcode this
	if !password.eq(the_password) {
		get_individual_channel_sender(&clients, &address)
			.send(InternalMessage {
				payload: InternalMessageAction::ResponseNotOkay(
					"The organizer password is incorrect.".to_owned(),
				),
				response_id,
				..Default::default()
			})
			.expect("Could not send NG as response");
		return;
	}

	let organizer = Organizer {
		name: random_word::gen(random_word::Lang::En).to_owned(),
	};

	clients.entry(address).and_modify(|c| {
		c.status = ClientStatus::Registered;
		c.organizer = Some(organizer.clone());
	});
	debug!("Marked client as organizer");

	get_individual_channel_sender(&clients, &address)
		.send(InternalMessage {
			payload: InternalMessageAction::ResponseOrganizerIdentity(organizer),
			response_id: response_id,
		})
		.expect("Could not send identity confirmation response");

	debug!("Response (organizer identity confirmation) sent");
}

fn retrieve_active_players(clients: &ClientsMap, address: SocketAddr) {
	debug!("===== Retrieve active players");

	let ics = get_individual_channel_sender(&clients, &address);
	let players = get_players(&clients);
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseActivePlayers(players),
		..Default::default()
	})
	.expect("Could not send list of active players");
}

fn exit_client(clients: &mut ClientsMap, address: SocketAddr) {
	debug!("===== Exit client");

	clients.remove(&address);
	debug!("Removed client: {}", address);

	announce_active_players(clients);
}

fn compile_choices(database: &DatabaseAccess, round: &Round) -> ChoicesMap {
	debug!("===== Compile choices");

	let mut choices: ChoicesMap = HashMap::new();

	let db_access = database
		.try_lock()
		.expect("Could not get access to database");
	match db_access.get_choices_by_round_id(round.id) {
		Ok(choices_for_active_round) => {
			choices = choices_for_active_round;
		}
		Err(_) => {}
	}

	choices
}

fn compile_game_state(database: &DatabaseAccess, clients: &ClientsMap) -> GameState {
	debug!("===== Compile game state");

	let players = get_players(clients);
	let round: Option<Round>;
	let mut choices: ChoicesMap = HashMap::new();

	{
		let db_access = database
			.try_lock()
			.expect("Could not get access to database");
		round = db_access
			.get_active_round()
			.expect("Could not query database for active round");
	}

	if round.is_some() {
		choices = compile_choices(database, round.as_ref().unwrap());
	}

	GameState {
		round,
		players,
		choices,
	}
}

fn retrieve_game_state(
	database: &DatabaseAccess,
	clients: &ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
) {
	debug!("===== Retrieve game state");

	let game_state = compile_game_state(database, clients);

	let ics = get_individual_channel_sender(&clients, &address);
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseGameState(game_state),
		response_id,
		..Default::default()
	})
	.expect("Could not send game state");
}

fn is_organizer(clients: &ClientsMap, address: &SocketAddr) -> bool {
	debug!("===== Is organizer?");

	let client = clients.get(address);
	if client.is_none() {
		return false;
	}
	let client = client.unwrap();

	return client.organizer.is_some();
}

fn is_player(clients: &ClientsMap, address: &SocketAddr) -> bool {
	debug!("===== Is player?");

	let client: Option<&Client> = clients.get(address);
	if client.is_none() {
		return false;
	}
	let client = client.unwrap();

	return client.player.is_some();
}

fn set_round(
	database: &DatabaseAccess,
	clients: &ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	round: Round,
) {
	debug!("===== Set round");

	if !is_organizer(clients, &address) {
		warn!("Set round request came from a non-organizer");
		return;
	}

	let db_access = database
		.try_lock()
		.expect("Could not get access to database");
	let find_existing_round = db_access
		.find_round_by_number_and_phase(round.number, round.phase)
		.expect("Could not query database to find round");

	let updated_round = match find_existing_round {
		Some(db_round) => db_access
			.update_round(
				db_round.number,
				db_round.phase,
				Some(round.state),
				Some(round.question),
				Some(round.choice_a),
				Some(round.choice_b),
			)
			.expect("Could not update round"),
		None => db_access
			.create_round(
				round.number,
				round.phase,
				round.state,
				round.question,
				round.choice_a,
				round.choice_b,
			)
			.expect("Could not create round"),
	};

	announce_round(database, clients, Some(updated_round.clone()));
	drop(db_access);

	if updated_round.state == RoundState::ShowVotes {
		announce_updated_choices(clients, compile_choices(database, &updated_round));
	}

	let ics = get_individual_channel_sender(&clients, &address);
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseOkay,
		response_id,
		..Default::default()
	})
	.expect("Could not send okay response");
}

fn get_active_round(database: &DatabaseAccess) -> Result<Round> {
	debug!("===== Get active round");

	let db_access = database
		.try_lock()
		.expect("Could not get access to database");
	let active_round = db_access.get_active_round()?;

	if active_round.is_none() {
		bail!("There is no active round");
	}

	Ok(active_round.unwrap())
}

fn announce_round(database: &DatabaseAccess, clients: &ClientsMap, round: Option<Round>) {
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

fn set_choice(
	database: &DatabaseAccess,
	clients: &ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	option: ChoiceOption,
) {
	debug!("===== Set choice");

	if !is_player(clients, &address) {
		warn!("Set choice request came from a non-player");
		return;
	}

	let db_access = database
		.try_lock()
		.expect("Could not get access to database");

	let round = db_access
		.get_active_round()
		.expect("Could not query database for active round");

	if round.is_none() {
		warn!("Trying to set choice when no round is active");
		return;
	}
	let round = round.unwrap();

	let player = clients
		.get(&address)
		.expect("Could not get client")
		.clone()
		.player
		.expect("Could not get player data");

	let check = db_access
		.check_player_is_allowed_to_vote(player.id)
		.expect("Could not query database for player data");

	if check == false {
		let ics = get_individual_channel_sender(&clients, &address);
		ics.send(InternalMessage {
			payload: InternalMessageAction::ResponseNotOkay(
				"You are not allowed to vote.".to_owned(),
			),
			response_id,
			..Default::default()
		})
		.expect("Could not send NG response");
		return;
	}

	let set_choice = db_access
		.update_or_create_choice(round.id, player.id, option)
		.expect("Could not set choice");

	announce_choice_to_organizers(clients, player, set_choice);

	let ics = get_individual_channel_sender(&clients, &address);
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseOkay,
		response_id,
		..Default::default()
	})
	.expect("Could not send okay response");
}

fn mark_choice_lie(
	database: &DatabaseAccess,
	clients: &ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	mark: MarkChoiceLie,
) {
	debug!("===== Mark choice");

	if !is_organizer(clients, &address) {
		warn!("Mark choice request came from a non-organizer");
		return;
	}

	let round = get_active_round(database);
	if round.is_err() {
		warn!("Could not get active round: {}", round.unwrap_err());
		return;
	}
	let round = round.unwrap();

	let db_access = database
		.try_lock()
		.expect("Could not get access to database");
	let update = db_access.mark_choice_lie(mark.id, mark.lie);
	if update.is_err() {
		warn!("Failed marking choice: {}", update.unwrap_err());
		return;
	}

	drop(db_access);
	announce_updated_choices(clients, compile_choices(database, &round));

	let ics = get_individual_channel_sender(&clients, &address);
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseOkay,
		response_id,
		..Default::default()
	})
	.expect("Could not send okay response");
}

fn announce_choice_to_organizers(clients: &ClientsMap, player: Player, choice: Choice) {
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

fn set_player(
	database: &DatabaseAccess,
	clients: &ClientsMap,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	player: Player,
) {
	debug!("===== Set player");

	if !is_organizer(clients, &address) {
		warn!("Set player request came from a non-organizer");
		return;
	}

	let db_access = database
		.try_lock()
		.expect("Could not get access to database");
	let updated_player = db_access.update_player(player.id, None, player.points, player.can_vote);
	if updated_player.is_err() {
		error!("Updating player error: {}", updated_player.unwrap_err());
		return;
	}
	let updated_player = updated_player.unwrap();

	announce_updated_player(clients, updated_player);

	let ics = get_individual_channel_sender(&clients, &address);
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseOkay,
		response_id,
		..Default::default()
	})
	.expect("Could not send okay response");
}
