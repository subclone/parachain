//! CLI configuration

use clap::Parser;
use op_core::postgres::PostgresConfig;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
	/// Path to the Postgres database
	#[arg(long, default_value = "localhost")]
	pub database_host: String,
	/// Port of the Postgres database
	#[arg(long, default_value = "5432")]
	pub database_port: u16,
	/// Username of the Postgres database
	#[arg(long, default_value = "postgres")]
	pub database_user: String,
	/// Name of the Postgres database
	#[arg(long, default_value = "postgres")]
	pub database_name: String,
	/// Substrate chain websocket endpoint
	#[arg(long, default_value = "ws://localhost:9944")]
	pub chain_endpoint: String,
	/// ISO-8583 specification file
	#[arg(long, default_value = "spec.yaml")]
	pub iso8583_spec: String,
	/// RPC port
	#[arg(long, default_value = "3030")]
	pub rpc_port: u16,
	/// Development mode
	#[arg(long)]
	pub dev: bool,
	/// Seed phrase for signing transactions
	#[arg(
		long,
		default_value = "0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a//Alice"
	)]
	pub seed: String,
	/// Substrate chain websocket endpoint
	#[arg(long, default_value = "ws://localhost:9944")]
	pub ws_url: String,
	/// OCW signer
	#[arg(
		long,
		default_value = "d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"
	)]
	pub ocw_signer: String,
}

impl Cli {
	/// Returns database URL
	pub fn get_db_url(&self) -> String {
		format!(
			"postgres://{}:{}@{}:{}/postgres",
			self.database_user, self.database_user, self.database_host, self.database_port
		)
	}

	/// Set env variables
	/// TODO: this is because of the weird way of how `iso8583-rs` loads the spec file
	pub fn set_env(&self) {
		std::env::set_var("SPEC_FILE", self.iso8583_spec.clone());
		// std::env::set_var("RUST_LOG", "info");
	}
}

#[allow(clippy::from_over_into)]
impl Into<PostgresConfig> for Cli {
	fn into(self) -> PostgresConfig {
		PostgresConfig {
			host: self.database_host,
			user: self.database_user,
			name: self.database_name,
			password: "postgres".to_string(),
			pool_max: 100,
		}
	}
}
