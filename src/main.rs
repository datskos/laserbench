mod args;
mod measure;
mod subscriber;

use crate::args::{App, Command};
use crate::measure::sub_blocks_and_process;
use clap::Parser;
use dotenvy::dotenv;
use eyre::eyre;
use std::env;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

fn get_connection_info(laser: bool) -> eyre::Result<(String, String)> {
    let (url_var, token_var) = if laser {
        ("LASERSTREAM_URL", "LASERSTREAM_TOKEN")
    } else {
        ("YELLOWSTONE_GRPC_URL", "YELLOWSTONE_GRPC_TOKEN")
    };

    let url = env::var(url_var)?;
    let token = env::var(token_var)?;

    if url.trim().is_empty() || token.trim().is_empty() {
        return Err(eyre!("{} or {} is empty", url_var, token_var));
    }

    Ok((url, token))
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let app = App::parse();
    dotenv().map_err(|e| eyre::eyre!("Could not read .env: {}", e))?;

    match app.command {
        Command::Record { kind, path } => {
            let (url, token) = get_connection_info(kind.laser)?;
            sub_blocks_and_process(url, token, path).await
        }
    }

    Ok(())
}
