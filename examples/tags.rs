use std::{boxed, error, result::Result};

use futures::stream::StreamExt;
use tracing::{error, info, warn};

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

  let image = match std::env::args().nth(2) {
    Some(x) => x,
    None => "library/debian".into(),
  };
  info!("[{registry}] requesting tags for image {image}");

  let user = std::env::var("DOCKER_REGISTRY_USER").ok();
  if user.is_none() {
    warn!("[{registry}] no $DOCKER_REGISTRY_USER for login user");
  }
  let password = std::env::var("DOCKER_REGISTRY_PASSWD").ok();
  if password.is_none() {
    warn!("[{registry}] no $DOCKER_REGISTRY_PASSWD for login password");
  }

  let res = run(&registry, user, password, &image).await;

  if let Err(e) = res {
    error!("[{}] {}", registry, e);
    std::process::exit(1);
  };
}

async fn run(
  host: &str,
  user: Option<String>,
  passwd: Option<String>,
  image: &str,
) -> Result<(), boxed::Box<dyn error::Error>> {
  let client = docker_registry::v2::Client::configure()
    .registry(host)
    .insecure_registry(false)
    .username(user)
    .password(passwd)
    .build()?;

  let login_scope = format!("repository:{image}:pull");

  let client = client.authenticate(&[&login_scope]).await?;

  client
    .get_tags(image, Some(7))
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .map(Result::unwrap)
    .for_each(|tag| {
      info!("{tag}");
    });

  Ok(())
}
