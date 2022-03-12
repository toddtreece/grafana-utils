use crate::grafana;
use anyhow::Result;
use clap::{App, AppSettings, Arg, ArgMatches};

pub async fn cli<'help>(_tags: &'help [&'help str]) -> App<'help> {
    App::new("start")
        .about("start grafana")
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::new("enterprise")
                .short('e')
                .about("Use grafana enterprise image")
                .takes_value(false),
        )
        .arg(
            Arg::new("logs")
                .short('l')
                .about("Tail the logs of the container")
                .takes_value(false),
        )
        .arg(
            Arg::new("random-port")
                .short('r')
                .about("Start grafana on a random open port")
                .takes_value(false),
        )
        .arg(
            Arg::new("proxy")
                .short('p')
                .about("Set HTTP_PROXY and HTTPS_PROXY environment variables")
                .takes_value(true),
        )
        .arg(
            Arg::new("version")
                .default_value("latest")
                .about("The grafana version to run")
                .index(1),
        )
}

pub async fn handle(matches: &ArgMatches) -> Result<()> {
    let version = matches.value_of("version").unwrap_or("latest");
    let enterprise = matches.is_present("enterprise");
    let random_port = matches.is_present("random-port");
    let logs = matches.is_present("logs");
    let proxy = matches.value_of("proxy");
    grafana::start(enterprise, version, random_port, logs, proxy).await
}
