use anyhow::Result;
use dirs::home_dir;
use futures::stream::StreamExt;
use shiplift::{
    rep::Container, tty::TtyChunk, ContainerOptions, Docker, Exec, ExecContainerOptions,
    LogsOptions, PullOptions,
};
use std::io::Write;
use std::net::TcpListener;

pub async fn get_image_tags() -> Result<Vec<String>> {
    let docker = Docker::new();
    let image = docker.images().get("grafana/grafana-enterprise");
    let details = image.inspect().await?;
    let tags = details.repo_tags.unwrap();
    Ok(tags)
}

fn get_image(enterprise: bool, version: &str) -> String {
    let mut image = "grafana/grafana".to_owned();
    if enterprise {
        image += "-enterprise";
    }
    image += ":";
    image += version;
    image
}

fn get_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

pub async fn get_containers(enterprise: bool, version: &str) -> Result<Vec<Container>> {
    let image = get_image(enterprise, version);
    let docker = Docker::new();
    let containers = docker.containers().list(&Default::default()).await?;
    let filtered: Vec<Container> = containers
        .into_iter()
        .filter(|c| image.eq(&c.image) || (version.eq("") && c.image.contains("grafana")))
        .collect();
    Ok(filtered)
}

pub async fn stop(enterprise: bool, version: &str) -> Result<()> {
    let docker = Docker::new();
    for c in get_containers(enterprise, version).await? {
        let container = docker.containers().get(&c.id);
        println!("stopping container {} - {}...", c.image, &container.id());
        container.kill(None).await?;
    }
    Ok(())
}

pub async fn reload_plugins() -> Result<()> {
    let docker = Docker::new();
    let opts = ExecContainerOptions::builder()
        .cmd(vec!["pkill", "datasource"])
        .attach_stdout(true)
        .attach_stderr(true)
        .build();
    for c in get_containers(false, "").await? {
        let exec = Exec::create(&docker, &c.id, &opts).await.unwrap();
        let mut stream = exec.start();
        while let Some(r) = stream.next().await {
            match r {
                Err(e) => eprintln!("Error: {}", e),
                _ => {}
            }
        }
    }
    Ok(())
}

pub async fn start(enterprise: bool, version: &str, random_port: bool, log: bool) -> Result<()> {
    let docker = Docker::new();
    let image = get_image(enterprise, version);

    println!("pulling docker image {}...", image);

    let mut stream = docker
        .images()
        .pull(&PullOptions::builder().image(&image).build());

    while let Some(pull_result) = stream.next().await {
        match pull_result {
            Err(e) => eprintln!("Error: {}", e),
            _ => {}
        }
    }

    let home = home_dir().unwrap().to_str().unwrap().to_owned();

    let mut plugins = home.clone();
    plugins.push_str("/plugins:/var/lib/grafana/plugins");

    let mut conf = home.clone();
    conf.push_str("/.grafana/grafana.ini:/etc/grafana/grafana.ini");

    let mut provisioning = home.clone();
    provisioning.push_str("/.grafana/provisioning:/etc/grafana/provisioning");

    let mut license = home.clone();
    if enterprise {
        license.push_str("/.grafana/ent-license.jwt:/var/lib/grafana/license.jwt");
    } else {
        license.push_str("/.grafana/license.jwt:/var/lib/grafana/license.jwt");
    }

    let mut port = 3000;

    if random_port {
        port = u32::from(get_port()?);
    }

    let mut options = ContainerOptions::builder(&image);

    options
        .expose(3000, "tcp", port)
        .expose(6060, "tcp", 6060)
        .env(vec![
            "GF_ENTERPRISE_LICENSE_PATH=/var/lib/grafana/license.jwt",
            "TERM=xterm-256color",
        ])
        .volumes(vec![&plugins, &conf, &provisioning, &license]);

    let info = docker.containers().create(&options.build()).await?;
    let container = docker.containers().get(&info.id);

    println!("running at http://localhost:{}/explore", port);
    container.start().await?;

    if log {
        logs(docker.clone(), container.id().to_string()).await;
    }

    Ok(())
}

async fn logs(docker: shiplift::Docker, id: String) {
    let container = docker.containers().get(id);
    let mut logs_stream = container.logs(
        &LogsOptions::builder()
            .follow(true)
            .stdout(true)
            .stderr(true)
            .build(),
    );

    while let Some(log_result) = logs_stream.next().await {
        match log_result {
            Ok(chunk) => print_chunk(chunk),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

fn print_chunk(chunk: TtyChunk) {
    match chunk {
        TtyChunk::StdOut(bytes) => {
            std::io::stdout().write_all(&bytes).unwrap();
        }
        TtyChunk::StdErr(bytes) => {
            std::io::stderr().write_all(&bytes).unwrap();
        }
        TtyChunk::StdIn(_) => unreachable!(),
    }
}
