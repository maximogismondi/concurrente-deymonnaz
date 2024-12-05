use std::env;

use actix::Actor;
use common::{config::Config, messages::concu_ride_messages::Initialize, Id};

mod gateway;
use gateway::Gateway;

fn read_args() -> Result<(Id, String), String> {
    let mut args = env::args();

    if args.len() != 3 {
        return Err("Usage: gateway <id> <config_path>".to_string());
    }

    args.next();

    let id = args.next().ok_or("ID not provided")?;
    let id = id.parse::<Id>().map_err(|_| "Invalid ID")?;

    let config_path = args.next().ok_or("Config path not provided")?;

    Ok((id, config_path))
}

fn start() -> Result<(), String> {
    let (_, config_path) = read_args()?;

    let config = Config::from_file(&config_path)?;
    let peers = config.peers();

    let gateway = Gateway::new(peers, config.gateway_accept_rate()).start();
    gateway.do_send(Initialize {});

    Ok(())
}

#[actix_rt::main]
async fn main() {
    match start() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    }

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
}
