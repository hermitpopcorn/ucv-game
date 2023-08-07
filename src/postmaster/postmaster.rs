use std::{net::SocketAddr, thread, time::Duration};

use crossbeam::channel::{unbounded, Sender};
use futures_util::{future, stream::SplitSink, SinkExt, StreamExt};
use log::{error, info};
use tokio::net::TcpStream;
use tokio_tungstenite::{
	accept_async,
	tungstenite::{
		Error as TungsteniteError, Message as TungsteniteMessage, Result as TungsteniteResult,
	},
	WebSocketStream,
};

use crate::gamemaster::types::{ChoiceOption, MarkChoiceLie, Player, Round};

use super::{
	json::{
		make_json_active_players, make_json_game_state, make_json_not_okay_response,
		make_json_okay_response, make_json_organizer_identity_response, make_json_player_choice,
		make_json_player_identity_response, make_json_round, make_json_updated_choices,
		make_json_updated_player, parse_message,
	},
	types::{
		InternalMessage, InternalMessageAction, ResponseIdentifier, WebSocketMessage,
		WebSocketMessageAction,
	},
};

pub fn accept_connection(peer: SocketAddr, stream: TcpStream, sender: Sender<InternalMessage>) {
	if let Err(e) = tokio::runtime::Runtime::new()
		.unwrap()
		.block_on(handle_connection(peer, stream, sender))
	{
		match e {
			TungsteniteError::ConnectionClosed
			| TungsteniteError::Protocol(_)
			| TungsteniteError::Utf8 => (),
			err => error!("Error processing connection: {}", err),
		}
	}
}

async fn handle_connection(
	address: SocketAddr,
	stream: TcpStream,
	gm_channel_sender: Sender<InternalMessage>,
) -> TungsteniteResult<()> {
	let ws_stream = accept_async(stream).await;
	if ws_stream.is_err() {
		return Err(TungsteniteError::ConnectionClosed);
	}
	let ws_stream = ws_stream.unwrap();
	info!("New WebSocket connection: {}", address);

	let (mut ws_sender, mut ws_receiver) = ws_stream.split();

	let (individual_channel_sender, individual_channel_receiver) = unbounded::<InternalMessage>();

	info!("Sending client registration: {}", address);
	gm_channel_sender
		.send(InternalMessage {
			payload: InternalMessageAction::RequestRegisterClient(
				address,
				individual_channel_sender.clone(),
			),
			..Default::default()
		})
		.expect("Could not send client registration message");

	info!("Awaiting client registration response: {}", address);
	let receive = individual_channel_receiver
		.recv()
		.expect("Could not receive client registration response");

	match receive.payload {
		InternalMessageAction::ResponseOkay => info!("Client registered successfully."),
		_ => panic!("Invalid response received from client registration process"),
	};

	loop {
		tokio::select! {
			socket_message = ws_receiver.next() => {
				match socket_message {
					Some(message) => {
						let message = message?;
						if message.is_close() {
							exit_client(&gm_channel_sender, address);
							break;
						}
						if !message.is_text() && !message.is_binary() {
							continue;
						}

						let message = parse_message(message.to_string());
						if message.is_none() {
							continue;
						}

						handle_message(&gm_channel_sender, address, message.unwrap());
					}
					None => continue,
				}
			}
			individual_channel_message = future::lazy(|_| individual_channel_receiver.try_recv()) => {
				if individual_channel_message.is_err() {
					thread::sleep(Duration::from_millis(50));
					continue;
				}

				let internal_message: InternalMessage = individual_channel_message.expect("Could not unwrap channel message");

				forward_message(&mut ws_sender, internal_message).await?;
			}
		}
	}

	Ok(())
}

fn handle_message(gmcs: &Sender<InternalMessage>, address: SocketAddr, message: WebSocketMessage) {
	match message.action {
		WebSocketMessageAction::LoginPlayer(name) => {
			log_in_player(gmcs, address, message.response_id, name)
		}
		WebSocketMessageAction::LoginOrganizer(password) => {
			log_in_organizer(gmcs, address, message.response_id, password)
		}

		WebSocketMessageAction::RetrieveGameState() => {
			retrieve_game_state(gmcs, address, message.response_id)
		}

		WebSocketMessageAction::SetRound(round) => {
			set_round(gmcs, address, message.response_id, round);
		}
		WebSocketMessageAction::MarkChoiceLie(mark) => {
			mark_choice_lie(gmcs, address, message.response_id, mark);
		}
		WebSocketMessageAction::SetChoiceOption(option) => {
			set_choice_option(gmcs, address, message.response_id, option);
		}
		WebSocketMessageAction::SetPlayer(player) => {
			set_player(gmcs, address, message.response_id, player)
		}
	};
}

async fn forward_message(
	wss: &mut SplitSink<WebSocketStream<TcpStream>, TungsteniteMessage>,
	internal_message: InternalMessage,
) -> TungsteniteResult<()> {
	let json: serde_json::Value = match internal_message.payload {
		InternalMessageAction::ResponseOkay => {
			make_json_okay_response(internal_message.response_id)
		}
		InternalMessageAction::ResponseNotOkay(message) => {
			make_json_not_okay_response(internal_message.response_id, message)
		}
		InternalMessageAction::ResponsePlayerIdentity(player) => {
			make_json_player_identity_response(internal_message.response_id, player)
		}
		InternalMessageAction::ResponseOrganizerIdentity(organizer) => {
			make_json_organizer_identity_response(internal_message.response_id, organizer)
		}
		InternalMessageAction::ResponseActivePlayers(active_players) => {
			make_json_active_players(internal_message.response_id, active_players)
		}
		InternalMessageAction::ResponseUpdatedPlayer(player) => {
			make_json_updated_player(internal_message.response_id, player)
		}
		InternalMessageAction::ResponseGameState(game_state) => {
			make_json_game_state(internal_message.response_id, game_state)
		}
		InternalMessageAction::ResponsePlayerChoice(player, choice) => {
			make_json_player_choice(internal_message.response_id, player, choice)
		}
		InternalMessageAction::ResponseRound(round) => {
			make_json_round(internal_message.response_id, round)
		}
		InternalMessageAction::ResponseUpdatedChoices(choices_map) => {
			make_json_updated_choices(internal_message.response_id, choices_map)
		}
		_ => return Ok(()),
	};

	wss.send(TungsteniteMessage::Text(json.to_string())).await
}

fn log_in_player(
	sender: &Sender<InternalMessage>,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	name: String,
) {
	let internal_message = InternalMessage {
		payload: InternalMessageAction::RequestRegisterActivePlayer(address, name),
		response_id,
		..Default::default()
	};
	sender
		.send(internal_message)
		.expect("Could not send request to GM for logging in player");
}

fn log_in_organizer(
	sender: &Sender<InternalMessage>,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	password: String,
) {
	let internal_message = InternalMessage {
		payload: InternalMessageAction::RequestRegisterOrganizer(address, password),
		response_id,
		..Default::default()
	};
	sender
		.send(internal_message)
		.expect("Could not send request to GM for logging in organizer");
}

fn exit_client(sender: &Sender<InternalMessage>, address: SocketAddr) {
	let internal_message = InternalMessage {
		payload: InternalMessageAction::ExitClient(address),
		..Default::default()
	};
	sender
		.send(internal_message)
		.expect("Could not send request to GM for goodbye sengen");
}

fn retrieve_game_state(
	sender: &Sender<InternalMessage>,
	address: SocketAddr,
	response_id: ResponseIdentifier,
) {
	let internal_message = InternalMessage {
		payload: InternalMessageAction::RequestGameState(address),
		response_id,
		..Default::default()
	};

	sender
		.send(internal_message)
		.expect("Could not send request to GM for game state");
}

fn set_round(
	sender: &Sender<InternalMessage>,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	round: Round,
) {
	let internal_message = InternalMessage {
		payload: InternalMessageAction::RequestSetRound(address, round),
		response_id,
		..Default::default()
	};

	sender
		.send(internal_message)
		.expect("Could not send request to GM for setting round");
}

fn set_choice_option(
	sender: &Sender<InternalMessage>,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	option: ChoiceOption,
) {
	let internal_message = InternalMessage {
		payload: InternalMessageAction::RequestSetChoiceOption(address, option),
		response_id,
		..Default::default()
	};

	sender
		.send(internal_message)
		.expect("Could not send request to GM for setting choice");
}

fn mark_choice_lie(
	sender: &Sender<InternalMessage>,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	mark: MarkChoiceLie,
) {
	let internal_message = InternalMessage {
		payload: InternalMessageAction::RequestMarkChoiceLie(address, mark),
		response_id,
		..Default::default()
	};

	sender
		.send(internal_message)
		.expect("Could not send request to GM for setting choice");
}

fn set_player(
	sender: &Sender<InternalMessage>,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	player: Player,
) {
	let internal_message = InternalMessage {
		payload: InternalMessageAction::RequestSetPlayerData(address, player),
		response_id,
		..Default::default()
	};

	sender
		.send(internal_message)
		.expect("Could not send request to GM for setting choice");
}
