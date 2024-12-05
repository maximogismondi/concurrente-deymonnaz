use actix::Actor;
use common::{config::Config, messages::concu_ride_messages::Initialize, Id};
use passenger::Passenger;
use std::env;

mod passenger;
mod passenger_status;

fn read_args() -> Result<(Id, String), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        return Err("Usage: passenger <id> <config_path>".to_string());
    }
    let id = args[1]
        .parse::<Id>()
        .map_err(|_| "Invalid ID".to_string())?;
    let config_path = args[2].clone();

    Ok((id, config_path))
}

fn start() -> Result<(), String> {
    let (id, config_path) = read_args()?;
    let config = Config::from_file(&config_path)?;
    let peers = config.peers();

    let passenger_config = config.passenger(id).ok_or("Passenger not found")?;
    let origin = passenger_config.origin.clone();
    let destination = passenger_config.destination.clone();

    let passenger = Passenger::new(id, origin, destination, peers).start();
    passenger.do_send(Initialize {});

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
