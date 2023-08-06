use std::{
	process::exit,
	sync::{Arc, Mutex},
	thread,
	time::Duration,
};

use crossbeam::channel::unbounded;
use database::sqlite::SqliteDatabase;
use env_logger::Env;
use futures_util::future;
use gamemaster::gamemaster::start_gamemaster;
use log::{info, warn};
use postmaster::{postmaster::accept_connection, types::InternalMessage};
use tokio::net::TcpListener;

mod database;
mod gamemaster;
mod postmaster;

#[tokio::main]
async fn main() {
	// Setup logger
	env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

	// Setup database
	let database = SqliteDatabase::new("database.db");
	let database_arc = Arc::new(Mutex::new(database));

	// Create crossbeam channels for communicating with gamemaster
	// Player <-> GM
	let (gm_channel_sender, gm_channel_receiver) = unbounded::<InternalMessage>();

	// Run gamemaster in new thread
	let gamemaster_handle = tokio::spawn(start_gamemaster(
		gm_channel_receiver.clone(),
		database_arc.clone(),
	));

	let addr = "0.0.0.0:9002";
	let listener = TcpListener::bind(&addr).await.expect("Can't listen");
	info!("Listening on: {}", addr);

	loop {
		tokio::select! {
			Ok((stream, _)) = listener.accept() => {
				let peer = stream.peer_addr().expect("connected streams should have a peer address");
				info!("Peer address: {}", peer);

				let cloned_gm_channel_sender = gm_channel_sender.clone();
				thread::spawn(move || { accept_connection(peer, stream, cloned_gm_channel_sender) });
			},
			gm_handle_finished = future::lazy(|_| gamemaster_handle.is_finished()) => {
				if !gm_handle_finished {
					thread::sleep(Duration::from_millis(100));
					continue;
				}

				warn!("Gamemaster thread is dead!");
				exit(1);
			}
		}
	}
}
