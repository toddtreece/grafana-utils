use crate::grafana;
use crate::plugin;
use anyhow::Result;
use clap::{App, AppSettings, ArgMatches};

pub async fn cli<'help>(_tags: &'help [&'help str]) -> App<'help> {
    App::new("build")
        .about("build the plugin in the current directory and reload it in the container")
        .setting(AppSettings::TrailingVarArg)
}

pub async fn handle(_matches: &ArgMatches) -> Result<()> {
    plugin::build();
    grafana::reload_plugins().await?;
    Ok(())
}
