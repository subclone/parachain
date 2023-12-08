use std::error::Error;

use deadpool_postgres::{Manager, Pool};
use log::info;
use tokio_postgres::{Config, NoTls};

use super::error::DomainError;

mod embedded {
	use refinery::embed_migrations;
	embed_migrations!("migrations");
}

/// Postgres database configuration
#[derive(Debug, Clone)]
pub struct PostgresConfig {
	pub host: String,
	pub user: String,
	pub password: String,
	pub name: String,
	pub pool_max: usize,
}

impl From<PostgresConfig> for tokio_postgres::Config {
	fn from(value: PostgresConfig) -> Self {
		let mut pg_config = tokio_postgres::Config::new();

		pg_config.user(&value.user);
		pg_config.password(&value.password);
		pg_config.dbname(&value.name);
		pg_config.host(&value.host);
		pg_config.ssl_mode(tokio_postgres::config::SslMode::Disable);
		pg_config.port(5432);

		pg_config
	}
}

/// Initializes Postgres database
pub fn init(postgres_config: PostgresConfig) -> Result<deadpool_postgres::Pool, Box<dyn Error>> {
	let pool_max = postgres_config.pool_max;
	let pg_config: tokio_postgres::Config = postgres_config.into();

	let mgr = Manager::new(pg_config, tokio_postgres::NoTls);

	Ok(Pool::builder(mgr).max_size(pool_max).build()?)
}

/// Runs database migrations
///
/// When run first time, it will create the tables for `BankAccount` and `Transaction`
pub async fn run_migrations(postgres_config: Config) -> Result<(), Box<dyn Error>> {
	info!("Running migrations");
	let (mut client, connection) = postgres_config.connect(tokio_postgres::NoTls).await?;

	let handler = tokio::spawn(async move {
		connection.await.unwrap();
	});

	let migration_report = embedded::migrations::runner().run_async(&mut client).await?;

	for migration in migration_report.applied_migrations() {
		info!("Migration Applied -  Name: {}, Version: {}", migration.name(), migration.version());
	}

	handler.abort();
	Ok(())
}

pub async fn mock_init(db_name: String) -> Result<Pool, DomainError> {
	use std::env;

	// This config is used for tests
	let postgres_config = PostgresConfig {
		host: env::var("POSTGRES_HOST").unwrap_or("localhost".to_string()),
		user: env::var("POSTGRES_USER").unwrap_or("postgres".to_string()),
		password: env::var("POSTGRES_PASSWORD").unwrap_or("postgres".to_string()),
		name: env::var("POSTGRES_DB_NAME").unwrap_or(db_name),
		pool_max: 100,
	};

	// This config is used for preparing the ground for tests
	// It resets the testing database and runs the migrations, since Postgres doesn't allow
	// dropping the database while there are active connections, we need to connect to the
	// default database and run the commands from there
	let mut pg_config = tokio_postgres::Config::new();
	pg_config.user(&postgres_config.user);
	pg_config.password(&postgres_config.password);
	pg_config.host(&postgres_config.host);

	let (client, connection) = pg_config.connect(NoTls).await?;

	let handler = tokio::spawn(async move {
		connection.await.unwrap();
	});

	let stmt = client.prepare("select datname from pg_database;").await?;
	let result = client.query(&stmt, &[]).await?;

	let mut databases: Vec<String> = vec![];
	result.into_iter().for_each(|row| databases.push(row.get("datname")));

	if databases.contains(&postgres_config.name) {
		let stmt = client.prepare(&format!("drop database {:?};", postgres_config.name)).await?;
		client.execute(&stmt, &[]).await?;
	}

	let stmt = client.prepare(&format!("create database {:?};", postgres_config.name)).await?;
	client.execute(&stmt, &[]).await?;

	handler.abort();

	// Now create the pool
	let test_pg_config: Config = postgres_config.into();

	let mgr = Manager::new(test_pg_config.clone(), tokio_postgres::NoTls);
	let pg_pool = Pool::builder(mgr).max_size(10).build()?;

	run_migrations(test_pg_config).await.expect("Error running migrations");

	Ok(pg_pool)
}
