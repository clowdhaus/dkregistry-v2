use std::{boxed, env, error, fs, io, str::FromStr};

use docker_registry::reference;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
    .pretty()
    .with_max_level(tracing::Level::INFO)
    .init();

  let dkr_ref = match std::env::args().nth(1) {
    Some(ref x) => reference::Reference::from_str(x),
    None => reference::Reference::from_str("quay.io/coreos/etcd"),
  }
  .unwrap();
  let registry = dkr_ref.registry();

  info!("[{registry}] downloading image {dkr_ref}");

  let mut user = None;
  let mut password = None;
  let home = dirs::home_dir().unwrap();
  let cfg = fs::File::open(home.join(".docker/config.json"));
  if let Ok(fp) = cfg {
    let creds = docker_registry::get_credentials(io::BufReader::new(fp), &registry);
    if let Ok(user_pass) = creds {
      user = user_pass.0;
      password = user_pass.1;
    } else {
      warn!("[{registry}] no credentials found in config.json");
    }
  } else {
    user = env::var("DOCKER_REGISTRY_USER").ok();
    if user.is_none() {
      warn!("[{registry}] no $DOCKER_REGISTRY_USER for login user");
    }
    password = env::var("DOCKER_REGISTRY_PASSWD").ok();
    if password.is_none() {
      warn!("[{registry}] no $DOCKER_REGISTRY_PASSWD for login password");
    }
  };

  let res = run(&dkr_ref, user, password).await;

  if let Err(e) = res {
    error!("[{}] {:?}", registry, e);
    std::process::exit(1);
  };
}

async fn run(
  dkr_ref: &reference::Reference,
  user: Option<String>,
  passwd: Option<String>,
) -> Result<(), boxed::Box<dyn error::Error>> {
  let image = dkr_ref.repository();
  let version = dkr_ref.version();

  let client = docker_registry::v2::Client::configure()
    .registry(&dkr_ref.registry())
    .insecure_registry(false)
    .username(user)
    .password(passwd)
    .build()?;

  let login_scope = "";

  let client = client.authenticate(&[login_scope]).await?;
  let manifest = client.get_manifest(&image, &version).await?;

  let layers_digests = manifest.layers_digests(None)?;
  info!("{} -> got {} layer(s)", &image, layers_digests.len(),);

  for layer_digest in &layers_digests {
    let blob = client.get_blob(&image, layer_digest).await?;
    info!("Layer {layer_digest}, got {} bytes.", blob.len());
  }

  info!("Downloaded {} layers", layers_digests.len());

  Ok(())
}
