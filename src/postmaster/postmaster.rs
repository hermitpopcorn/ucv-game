use std::net::SocketAddr;

use crossbeam::channel::{unbounded, Sender};
use futures_util::{future, SinkExt, StreamExt};
use log::{error, info};
use tokio::net::TcpStream;
use tokio_tungstenite::{
	accept_async,
	tungstenite::{
		Error as TungsteniteError, Message as TungsteniteMessage, Result as TungsteniteResult,
	},
};

use super::{
	json::{
		make_json_active_players, make_json_not_okay_response, make_json_okay_response,
		make_json_player_identity_response, parse_message,
	},
	types::{
		InternalMessage, InternalMessageAction, ResponseIdentifier, WebSocketMessage,
		WebSocketMessageAction,
	},
};

pub async fn accept_connection(
	peer: SocketAddr,
	stream: TcpStream,
	sender: Sender<InternalMessage>,
) {
	if let Err(e) = handle_connection(peer, stream, sender).await {
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
	let ws_stream = accept_async(stream).await.expect("Failed to accept");
	info!("New WebSocket connection: {}", address);

	let (mut ws_sender, mut ws_receiver) = ws_stream.split();

	let (individual_channel_sender, individual_channel_receiver) = unbounded::<InternalMessage>();

	gm_channel_sender
		.send(InternalMessage {
			payload: InternalMessageAction::RegisterClient(
				address,
				individual_channel_sender.clone(),
			),
			..Default::default()
		})
		.expect("Could not send client registration message");

	let receive = individual_channel_receiver
		.recv()
		.expect("Could not receive client registration response");

	match receive.payload {
		InternalMessageAction::ResponseOkay => (),
		_ => panic!("Invalid response received from client registration process"),
	};

	loop {
		tokio::select! {
			socket_message = ws_receiver.next() => {
				match socket_message {
					Some(message) => {
						let message = message?;
						if message.is_text() || message.is_binary() {
							let message = parse_message(message.to_string());
							if message.is_none() {
								continue;
							}

							handle_message(&gm_channel_sender, address, message.unwrap());
						} else if message.is_close() {
							exit_client(&gm_channel_sender, address);
							break;
						}
					}
					None => continue,
				}
			}
			individual_channel_message = future::lazy(|_| individual_channel_receiver.try_recv()) => {
				if individual_channel_message.is_err() {
					continue;
				}

				let channel_message: InternalMessage = individual_channel_message.expect("Could not unwrap channel message");

				let json: serde_json::Value = match channel_message.payload {
					InternalMessageAction::ResponseOkay => {
						make_json_okay_response(channel_message.response_id)
					},
					InternalMessageAction::ResponseNotOkay(message) => {
						make_json_not_okay_response(channel_message.response_id, message)
					},
					InternalMessageAction::ResponsePlayerIdentity(player) => {
						make_json_player_identity_response(channel_message.response_id, player)
					},
					InternalMessageAction::ResponseActivePlayers(active_players) => {
						make_json_active_players(channel_message.response_id, active_players)
					},
					_ => continue,
				};

				ws_sender.send(TungsteniteMessage::Text(json.to_string())).await?;
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
	};
}

fn log_in_player(
	sender: &Sender<InternalMessage>,
	address: SocketAddr,
	response_id: ResponseIdentifier,
	name: String,
) {
	let internal_message = InternalMessage {
		payload: InternalMessageAction::RegisterActivePlayer(address, response_id, name),
		..Default::default()
	};
	sender
		.send(internal_message)
		.expect("Could not send request to GM for logging in player");
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
