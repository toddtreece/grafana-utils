use anyhow::Result;
use clap::{App, AppSettings};
use clap_generate::{generate, generators::Zsh};

use cli::completion;

mod cli;
mod grafana;
mod plugin;

#[tokio::main]
async fn main() -> Result<()> {
    let resp = grafana::get_image_tags().await?;
    let tags: Vec<&str> = resp.iter().map(String::as_ref).collect();

    let mut app = App::new("Grafana Utils")
        .setting(AppSettings::TrailingVarArg)
        .bin_name("grf")
        .subcommand(cli::start::cli(&tags).await)
        .subcommand(cli::stop::cli(&tags).await)
        .subcommand(cli::plugin::cli(&tags).await)
        .subcommand(completion::cli());

    let matches = app.clone().get_matches();

    match matches.subcommand() {
        Some(("start", sub_matches)) => cli::start::handle(sub_matches).await?,
        Some(("stop", sub_matches)) => cli::stop::handle(sub_matches).await?,
        Some(("build", sub_matches)) => cli::plugin::handle(sub_matches).await?,
        Some(("completion", _)) => generate::<Zsh, _>(&mut app, "grf", &mut std::io::stdout()),
        _ => println!("{}", app.generate_usage()),
    }

    Ok(())
}
