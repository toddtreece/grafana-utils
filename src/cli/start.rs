use crate::grafana;
use anyhow::Result;
use clap::{App, AppSettings, Arg, ArgMatches};

pub async fn cli<'help>(tags: &'help [&'help str]) -> App<'help> {
    App::new("start")
        .about("start grafana")
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::new("enterprise")
                .short('e')
                .about("Use grafana enterprise container")
                .takes_value(false),
        )
        .arg(
            Arg::new("random-port")
                .short('r')
                .about("Use grafana enterprise container")
                .takes_value(false),
        )
        .arg(
            Arg::new("version")
                .default_value("latest")
                .about("the grafana version to run")
                .index(1),
        )
}

pub async fn handle(matches: &ArgMatches) -> Result<()> {
    let version = matches.value_of("version").unwrap_or("latest");
    let enterprise = matches.is_present("enterprise");
    let random_port = matches.is_present("random-port");
    grafana::start(enterprise, version, random_port).await
}
