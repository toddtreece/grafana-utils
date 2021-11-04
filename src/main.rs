use anyhow::Result;
use clap::{App, AppSettings};
use clap_generate::{generate, generators::Zsh};

use cli::completion;
use cli::start;
use cli::stop;

mod cli;
mod grafana;

#[tokio::main]
async fn main() -> Result<()> {
    let resp = grafana::get_image_tags().await?;
    let tags: Vec<&str> = resp.iter().map(String::as_ref).collect();

    let mut app = App::new("Grafana Utils")
        .setting(AppSettings::TrailingVarArg)
        .bin_name("grf")
        .subcommand(start::cli(&tags).await)
        .subcommand(stop::cli(&tags).await)
        .subcommand(completion::cli());

    let matches = app.clone().get_matches();

    match matches.subcommand() {
        Some(("start", sub_matches)) => start::handle(sub_matches).await?,
        Some(("stop", sub_matches)) => stop::handle(sub_matches).await?,
        Some(("completion", _)) => generate::<Zsh, _>(&mut app, "grf", &mut std::io::stdout()),
        _ => println!("{}", app.generate_usage()),
    }

    Ok(())
}
