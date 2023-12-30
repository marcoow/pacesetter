use crate::cli::ui::{log, LogType};
use clap::{arg, Command};
use cruet::{
    case::title::to_title_case,
    string::{pluralize::to_plural, singularize::to_singular},
};
use std::fs::File;
use std::time::SystemTime;

fn commands() -> Command {
    Command::new("db")
        .about("A CLI tool to generate project files.")
        .subcommand_required(true)
        .subcommand(
            Command::new("migration")
                .about("Generate a new migration file")
                .arg(arg!([NAME]).required(true)),
        )
        .subcommand(
            Command::new("entity")
                .about("Generate a new entity")
                .arg(arg!([NAME]).required(true)),
        )
}

pub async fn cli() {
    let matches = commands().get_matches();

    match matches.subcommand() {
        Some(("migration", sub_matches)) => {
            let name = sub_matches
                .get_one::<String>("NAME")
                .map(|s| s.as_str())
                .unwrap();
            generate_migration(name).await;
        }
        Some(("entity", sub_matches)) => {
            let name = sub_matches
                .get_one::<String>("NAME")
                .map(|s| s.as_str())
                .unwrap();
            generate_entity(name).await;
        }
        _ => unreachable!(),
    }
}

async fn generate_migration(name: &str) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let name = format!("V{}__{}.sql", timestamp.as_secs(), name);
    match File::create(format!("./db/migrations/{}", name)) {
        Ok(_) => log(
            LogType::Success,
            format!("Created migration {}.", name).as_str(),
        ),
        Err(_) => log(LogType::Error, "Could not create migration file!"),
    }
}

async fn generate_entity(name: &str) {
    let name = to_singular(name).to_lowercase();
    let name_plural = to_plural(&name);
    match File::create(format!("./db/src/entities/{}.rs", name_plural)) {
        Ok(_) => log(
            LogType::Success,
            format!("Created entity {}.", to_title_case(&name)).as_str(),
        ),
        Err(_) => log(LogType::Error, "Could not create entity!"),
    }
}
