use std::sync::{Arc, Mutex};

use crossbeam::channel::unbounded;
use database::sqlite::SqliteDatabase;
use env_logger::Env;
use gamemaster::start_gamemaster;
use log::info;
use postmaster::postmaster::accept_connection;
use tokio::{net::TcpListener, spawn};
use types::ChannelMessage;

mod database;
mod gamemaster;
mod postmaster;
mod types;

#[tokio::main]
async fn main() {
	// Setup logger
	env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

	// Setup database
	let database = SqliteDatabase::new("database.db");
	let database_arc = Arc::new(Mutex::new(database));

	// Create crossbeam channel for communicating with gamemaster
	let (gm_channel_sender, gm_channel_receiver) = unbounded::<ChannelMessage>();

	// Run gamemaster in new thread
	let _gamemaster_handle = spawn(start_gamemaster(
		gm_channel_receiver.clone(),
		database_arc.clone(),
	));

	let addr = "127.0.0.1:9002";
	let listener = TcpListener::bind(&addr).await.expect("Can't listen");
	info!("Listening on: {}", addr);

	while let Ok((stream, _)) = listener.accept().await {
		let peer = stream
			.peer_addr()
			.expect("connected streams should have a peer address");
		info!("Peer address: {}", peer);

		spawn(accept_connection(peer, stream, gm_channel_sender.clone()));
	}
}
