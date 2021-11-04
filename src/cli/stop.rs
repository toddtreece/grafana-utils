use crate::grafana;
use anyhow::Result;
use clap::{App, AppSettings, Arg, ArgMatches};

pub async fn cli<'help>(tags: &'help [&'help str]) -> App<'help> {
    App::new("stop")
        .about("stop running version")
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::new("version")
                .about("the grafana version to stop")
                .index(1),
        )
        .arg(
            Arg::new("enterprise")
                .short('e')
                .about("Use grafana enterprise container")
                .takes_value(false),
        )
}

pub async fn handle(matches: &ArgMatches) -> Result<()> {
    let version = matches.value_of("version").unwrap_or("");
    let enterprise = matches.is_present("enterprise");
    grafana::stop(enterprise, version).await
}
