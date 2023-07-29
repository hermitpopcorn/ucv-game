use std::{collections::HashMap, net::SocketAddr};

use anyhow::{Ok, Result};
use crossbeam::channel::{Receiver, Sender};
use log::{debug, warn};

use crate::{
	database::database::DatabaseAccess,
	postmaster::types::{InternalMessage, InternalMessageAction, ResponseIdentifier},
};

use super::types::{Client, ClientsMap, Player};

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
			InternalMessageAction::RegisterActivePlayer(address, response_id, name) => {
				register_active_player(&database, &mut clients, address, response_id, name);
			}
			InternalMessageAction::RetrieveActivePlayers(address) => {
				retrieve_active_players(&clients, address);
			}
			InternalMessageAction::ExitClient(address) => {
				exit_client(&mut clients, address);
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
	let client = clients_map
		.get(address)
		.expect("Could not find the client when searching for individual channel");
	&client.individual_channel_sender
}

fn get_players(clients_map: &ClientsMap) -> Vec<Player> {
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
			player: None,
		},
	);

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

	// Get lock to database
	let db_access = database.lock().expect("Could not get access to database");

	// Find player data in database
	debug!("Finding/creating player ({})...", &name);
	let player = db_access
		.find_or_create_player(&name)
		.expect(&format!("Could not find or create player: {}.", &name));

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
				.expect("Could not send NG as response");
			return;
		}
	}

	clients
		.entry(address)
		.and_modify(|c| c.player = Some(player.clone()));
	debug!("Connected players updated");

	let ics = get_individual_channel_sender(&clients, &address);

	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseIdentity(player),
		response_id,
		..Default::default()
	})
	.expect("Could not send identity confirmation response");
	debug!("Response (identity confirmation) sent");

	let players = get_players(&clients);
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseActivePlayers(players),
		..Default::default()
	})
	.expect("Could not send list of active players as response");
	debug!("Response (list of connected players) sent");
}

fn retrieve_active_players(clients: &ClientsMap, address: SocketAddr) {
	let ics = get_individual_channel_sender(&clients, &address);
	let players = get_players(&clients);
	ics.send(InternalMessage {
		payload: InternalMessageAction::ResponseActivePlayers(players),
		..Default::default()
	})
	.expect("Could not send list of active players");
}

fn exit_client(clients: &mut ClientsMap, address: SocketAddr) {
	clients.remove(&address);
	debug!("Removed client: {}", address);
}
