use std::{boxed, error};

use tracing::{error, info};

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
    .pretty()
    .with_max_level(tracing::Level::INFO)
    .init();

  let registry = match std::env::args().nth(1) {
    Some(x) => x,
    None => "registry-1.docker.io".into(),
  };

  let res = run(&registry).await;

  if let Err(e) = res {
    error!("[{registry}] {e}");
    std::process::exit(1);
  };
}

async fn run(host: &str) -> Result<bool, boxed::Box<dyn error::Error>> {
  let client = docker_registry::v2::Client::configure()
    .registry(host)
    .insecure_registry(false)
    .build()?;

  let supported = client.is_v2_supported().await?;
  if supported {
    info!("{host} supports v2");
  } else {
    info!("{host} does NOT support v2");
  }
  Ok(supported)
}
