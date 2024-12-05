use actix::Actor;
use common::{config::Config, messages::concu_ride_messages::Initialize, Id};

use std::env;

mod driver;

use driver::Driver;

fn read_args() -> Result<(Id, String), String> {
    let mut args = env::args();

    if args.len() != 3 {
        return Err("Usage: driver <id> <config_path>".to_string());
    }

    args.next();

    let id = args.next().ok_or("ID not provided")?;
    let id = id.parse::<Id>().map_err(|_| "Invalid ID")?;

    let config_path = args.next().ok_or("Config path not provided")?;

    Ok((id, config_path))
}

fn start() -> Result<(), String> {
    let (id, config_path) = read_args()?;

    let config = Config::from_file(&config_path)?;
    let peers = config.peers();
    let driver_config = config.driver(id).ok_or("Driver not found")?;
    let position = driver_config.position;
    let driver_accept_rate = config.driver_accept_rate();

    let driver = Driver::new(id, position, peers, driver_accept_rate).start();
    driver.do_send(Initialize {});

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
