use anyhow::Context;
use clap::{Parser, Subcommand};
use {{crate_name}}_cli::util::ui::UI;
use {{crate_name}}_config::{load_config, parse_env, Config, Environment};
use {{crate_name}}_config::DatabaseConfig;
use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{
    migrate::{Migrate, Migrator},
    ConnectOptions, Connection, Executor,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use url::Url;

#[tokio::main]
async fn main() {
    cli().await;
}

#[derive(Parser)]
#[command(author, version, about = "A CLI tool to manage the project's database.", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, help = "Choose the environment (development, test, production).", value_parser = parse_env, default_value = "development")]
    env: Environment,

    #[arg(long, global = true, help = "Disable colored output.")]
    no_color: bool,

    #[arg(long, global = true, help = "Enable debug output.")]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Drop the database")]
    Drop,
    #[command(about = "Create the database")]
    Create,
    #[command(about = "Migrate the database")]
    Migrate,
    #[command(about = "Reset (drop, create, migrate) the database")]
    Reset,
    #[command(about = "Seed the database")]
    Seed,
}

#[allow(missing_docs)]
async fn cli() {
    let cli = Cli::parse();

    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let mut ui = UI::new(&mut stdout, &mut stderr, !cli.no_color, cli.debug);

    let config: Result<Config, anyhow::Error> = load_config(&cli.env);
    match config {
        Ok(config) => match cli.command {
            Commands::Drop => {
                ui.info(&format!("Dropping {} database…", &cli.env));
                match drop(&config.database).await {
                    Ok(db_name) => {
                        ui.success(&format!("Dropped database {} successfully.", &db_name))
                    }
                    Err(e) => ui.error("Could not drop database!", e),
                }
            }
            Commands::Create => {
                ui.info(&format!("Creating {} database…", &cli.env));
                match create(&config.database).await {
                    Ok(db_name) => {
                        ui.success(&format!("Created database {} successfully.", &db_name))
                    }
                    Err(e) => ui.error("Could not create database!", e),
                }
            }
            Commands::Migrate => {
                ui.info(&format!("Migrating {} database…", &cli.env));
                ui.indent();
                match migrate(&mut ui, &config.database).await {
                    Ok(migrations) => {
                        ui.outdent();
                        ui.success(&format!("{} migrations applied.", migrations));
                    }
                    Err(e) => {
                        ui.outdent();
                        ui.error("Could not migrate database!", e);
                    }
                }
            }
            Commands::Seed => {
                ui.info(&format!("Seeding {} database…", &cli.env));
                match seed(&config.database).await {
                    Ok(_) => ui.success("Seeded database successfully."),
                    Err(e) => ui.error("Could not seed database!", e),
                }
            }
            Commands::Reset => {
                ui.info(&format!("Resetting {} database…", &cli.env));
                ui.indent();
                match reset(&mut ui, &config.database).await {
                    Ok(db_name) => {
                        ui.outdent();
                        ui.success(&format!("Reset database {} successfully.", db_name));
                    }
                    Err(e) => {
                        ui.outdent();
                        ui.error("Could not reset database!", e)
                    }
                }
            }
        },
        Err(e) => ui.error("Could not load config!", e),
    }
}

async fn drop(config: &DatabaseConfig) -> Result<String, anyhow::Error> {
    let db_config = get_db_config(config);
    let db_name = db_config
        .get_database()
        .context("Failed to get database name!")?;
    let mut root_connection = get_root_db_client(config).await;

    let query = format!("DROP DATABASE {}", db_name);
    root_connection
        .execute(query.as_str())
        .await
        .context("Failed to drop database!")?;

    Ok(String::from(db_name))
}

async fn create(config: &DatabaseConfig) -> Result<String, anyhow::Error> {
    let db_config = get_db_config(config);
    let db_name = db_config
        .get_database()
        .context("Failed to get database name!")?;
    let mut root_connection = get_root_db_client(config).await;

    let query = format!("CREATE DATABASE {}", db_name);
    root_connection
        .execute(query.as_str())
        .await
        .context("Failed to create database!")?;

    Ok(String::from(db_name))
}

async fn migrate(ui: &mut UI<'_>, config: &DatabaseConfig) -> Result<i32, anyhow::Error> {
    let db_config = get_db_config(config);
    let migrator = Migrator::new(Path::new("db/migrations"))
        .await
        .context("Failed to create migrator!")?;
    let mut connection = db_config
        .connect()
        .await
        .context("Failed to connect to database!")?;

    connection
        .ensure_migrations_table()
        .await
        .context("Failed to ensure migrations table!")?;

    let applied_migrations: HashMap<_, _> = connection
        .list_applied_migrations()
        .await
        .context("Failed to list applied migrations!")?
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    let mut applied = 0;
    for migration in migrator.iter() {
        if applied_migrations.get(&migration.version).is_none() {
            connection
                .apply(migration)
                .await
                .context("Failed to apply migration {}!")?;
            ui.log(&format!("Applied migration {}.", migration.version));
            applied += 1;
        }
    }

    Ok(applied)
}

async fn seed(config: &DatabaseConfig) -> Result<(), anyhow::Error> {
    let mut connection = get_db_client(config).await;

    let statements = fs::read_to_string("./db/seeds.sql")
        .expect("Could not read seeds – make sure db/seeds.sql exists!");

    let mut transaction = connection
        .begin()
        .await
        .context("Failed to start transaction!")?;
    transaction
        .execute(statements.as_str())
        .await
        .context("Failed to execute seeds!")?;

    Ok(())
}

async fn reset(ui: &mut UI<'_>, config: &DatabaseConfig) -> Result<String, anyhow::Error> {
    ui.log("Dropping database…");
    drop(config).await?;
    ui.log("Recreating database…");
    let db_name = create(config).await?;
    ui.log("Migrating database…");
    ui.indent();
    let migration_result = migrate(ui, config).await;
    ui.outdent();

    match migration_result {
        Ok(_) => Ok(db_name),
        Err(e) => Err(e),
    }
}

fn get_db_config(config: &DatabaseConfig) -> PgConnectOptions {
    let db_url = Url::parse(&config.url).expect("Invalid DATABASE_URL!");
    ConnectOptions::from_url(&db_url).expect("Invalid DATABASE_URL!")
}

async fn get_db_client(config: &DatabaseConfig) -> PgConnection {
    let db_config = get_db_config(config);
    let connection: PgConnection = Connection::connect_with(&db_config).await.unwrap();

    connection
}

async fn get_root_db_client(config: &DatabaseConfig) -> PgConnection {
    let db_config = get_db_config(config);
    let root_db_config = db_config.clone().database("postgres");
    let connection: PgConnection = Connection::connect_with(&root_db_config).await.unwrap();

    connection
}
