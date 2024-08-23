use std::{boxed, env, error, fs, io, path::Path, result::Result};

use docker_registry::render;
use futures::future::try_join_all;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), boxed::Box<dyn error::Error>> {
  let registry = match std::env::args().nth(1) {
    Some(x) => x,
    None => "quay.io".into(),
  };

  let image = match std::env::args().nth(2) {
    Some(x) => x,
    None => "coreos/etcd".into(),
  };

  let version = match std::env::args().nth(3) {
    Some(x) => x,
    None => "latest".into(),
  };

  let path_string = format!("{}:{}", &image, &version).replace("/", "_");
  let path = Path::new(&path_string);
  if path.exists() {
    let msg = format!("path {:?} already exists", &path);
    match std::env::var("IMAGE_OVERWRITE") {
      Ok(value) if value == "true" => {
        std::fs::remove_dir_all(path)?;
        eprintln!("{msg}, removing.");
      }
      _ => return Err(format!("{msg}, exiting.").into()),
    }
  };

  info!("[{registry}] downloading image {image}:{version}");

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

  let res = run(&registry, &image, &version, user, password, path).await;

  if let Err(e) = res {
    error!("[{registry}] {e}");
    std::process::exit(1);
  };

  Ok(())
}

async fn run(
  registry: &str,
  image: &str,
  version: &str,
  user: Option<String>,
  passwd: Option<String>,
  path: &Path,
) -> Result<(), boxed::Box<dyn error::Error>> {
  tracing_subscriber::fmt()
    .pretty()
    .with_max_level(tracing::Level::INFO)
    .init();

  let client = docker_registry::v2::Client::configure()
    .registry(registry)
    .insecure_registry(false)
    .username(user)
    .password(passwd)
    .build()?;

  let login_scope = format!("repository:{image}:pull");

  let client = client.authenticate(&[&login_scope]).await?;
  let manifest = client.get_manifest(image, version).await?;
  let layers_digests = manifest.layers_digests(None)?;

  info!("{} -> got {} layer(s)", &image, layers_digests.len(),);

  let blob_futures = layers_digests
    .iter()
    .map(|layer_digest| client.get_blob(image, layer_digest))
    .collect::<Vec<_>>();

  let blobs = try_join_all(blob_futures).await?;

  println!("Downloaded {} layers", blobs.len());
  tokio::fs::create_dir(path).await?;
  let can_path = path.canonicalize()?;

  info!("Unpacking layers to {:?}", &can_path);
  render::unpack(&blobs, &can_path)?;
  Ok(())
}
