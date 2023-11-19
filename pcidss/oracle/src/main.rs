//! Main entrypoint of the oracle

use clap::Parser;
use dotenv::dotenv;
use op_core::postgres::{self, run_migrations, PostgresConfig};
use std::{io, sync::Arc};

pub mod cli;
pub mod services;
pub mod types;

use services::rpc;

#[cfg(test)]
mod tests;

/// Main entrypoint of the oracle
#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    let args = cli::Cli::parse();
    // TODO: this is because of the weird way of how `iso8583-rs` loads the spec file
    args.set_env();

    env_logger::init();

    log::info!("Starting PCIDSS Gateway Oracle");

    let db_config: PostgresConfig = args.clone().into();

    log::info!("Connecting to Postgres database: {}", args.get_db_url());

    let pg_pool_result = postgres::init(db_config.clone());

    // run migrations
    if let Err(e) = run_migrations(db_config.into()).await {
        log::error!("Could not run migrations {:?}", e);
        std::process::exit(1)
    }

    if pg_pool_result.is_err() {
        log::error!(
            "Could not initialize Postgres DB: {}",
            pg_pool_result.unwrap_err()
        );
        std::process::exit(1)
    }

    log::info!("Connected to Postgres database");

    let pg_pool = Arc::new(pg_pool_result.unwrap());

    let pg_pool_move = pg_pool.clone();

    tokio::spawn(async move {
        let result = rpc::run(args.rpc_port, pg_pool_move, args.dev).await;
        if result.is_err() {
            log::error!("Could not start RPC: {}", result.unwrap_err().to_string());
            std::process::exit(1)
        }
    });

    op_core::utils::block_until_sigint().await;

    Ok(())
}
