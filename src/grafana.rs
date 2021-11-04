use anyhow::Result;
use dirs::home_dir;
use futures::StreamExt;
use shiplift::{ContainerOptions, Docker, PullOptions};
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

pub async fn stop(enterprise: bool, version: &str) -> Result<()> {
    let image = get_image(enterprise, version);
    let docker = Docker::new();
    let containers = docker.containers().list(&Default::default()).await?;
    for c in containers {
        if image.eq(&c.image) || version.eq("") {
            let container = docker.containers().get(&c.id);
            println!("stopping container {} - {}...", image, &container.id());
            container.kill(None).await?;
        }
    }
    Ok(())
}

pub async fn start(enterprise: bool, version: &str, random_port: bool) -> Result<()> {
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
        ])
        .volumes(vec![&plugins, &conf, &provisioning, &license]);

    let info = docker.containers().create(&options.build()).await?;
    let container = docker.containers().get(&info.id);

    println!("running at http://localhost:{}/explore", port);
    container.start().await?;

    Ok(())
}
