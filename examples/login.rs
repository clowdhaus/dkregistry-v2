use std::{boxed, error, result::Result};

use tracing::{error, warn};

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

  let login_scope = match std::env::args().nth(2) {
    Some(x) => x,
    None => "".into(),
  };

  let user = std::env::var("DOCKER_REGISTRY_USER").ok();
  if user.is_none() {
    warn!("[{registry}] no $DOCKER_REGISTRY_USER for login user");
  }
  let password = std::env::var("DOCKER_REGISTRY_PASSWD").ok();
  if password.is_none() {
    warn!("[{registry}] no $DOCKER_REGISTRY_PASSWD for login password");
  }

  let res = run(&registry, user, password, login_scope).await;

  if let Err(e) = res {
    error!("[{registry}] {e}");
    std::process::exit(1);
  };
}

async fn run(
  host: &str,
  user: Option<String>,
  passwd: Option<String>,
  login_scope: String,
) -> Result<(), boxed::Box<dyn error::Error>> {
  let client = docker_registry::v2::Client::configure()
    .registry(host)
    .insecure_registry(false)
    .username(user)
    .password(passwd)
    .build()?;

  let client = client.authenticate(&[&login_scope]).await?;
  client.is_auth().await?;
  Ok(())
}
