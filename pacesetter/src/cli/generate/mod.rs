use crate::cli::ui::{log, LogType};
use anyhow::Context;
use clap::{arg, Command};
use cruet::{
    case::title::to_title_case,
    string::{pluralize::to_plural, singularize::to_singular},
};
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::time::SystemTime;

static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("VERGEN_GIT_SHA"), ")");

static BLUEPRINTS_DIR: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/cli/generate/blueprints");

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
    let struct_name = to_title_case(&name);

    let tmp_directory = std::env::temp_dir().join(format!("pacesetter-blueprint-{}", VERSION));
    std::fs::create_dir_all(&tmp_directory)
        .context("Failed to create a temporary directory for Pacesetter's blueprints")
        .unwrap();
    BLUEPRINTS_DIR
        .extract(&tmp_directory)
        .context("Failed to extract Pacesetter's blueprints to a temporary directory")
        .unwrap();
    let blueprint_path = tmp_directory.join("entity").join("file.rs.liquid");
    let blueprint_path = blueprint_path
        .to_str()
        .unwrap_or("Failed to get full path to Pacesetter's blueprint");
    let template_source =
        fs::read_to_string(blueprint_path).expect("Should have been able to read the file");
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(&template_source)
        .unwrap();
    let variables = liquid::object!({
        "entity_struct_name": struct_name,
        "entity_singular_name": name,
        "entity_plural_name": name_plural,
    });
    let output = template.render(&variables).unwrap();

    match File::create(format!("./db/src/entities/{}.rs", name_plural)) {
        Ok(mut file) => match file.write_all(output.as_bytes()) {
            Ok(_) => log(
                LogType::Success,
                format!("Created entity {}.", struct_name).as_str(),
            ),
            Err(_) => log(LogType::Error, "Could not create entity!"),
        },
        Err(_) => log(LogType::Error, "Could not create entity!"),
    }

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("./db/src/entities/mod.rs")
        .unwrap();

    if writeln!(file, "pub mod {};", name_plural).is_err() {
        log(
            LogType::Error,
            &format!(
                r#"Could not add mod declaration – add "pub mod {};" to ./db/src/entities/mod.rs"!"#,
                name_plural
            ),
        );
    }
}
