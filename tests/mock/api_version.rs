static API_VERSION_K: &str = "Docker-Distribution-API-Version";
static API_VERSION_V: &str = "registry/2.0";

#[tokio::test]
async fn test_version_check_status_ok() {
  let mut server = mockito::Server::new_async().await;
  let addr = server.host_with_port();

  let mock = server
    .mock("GET", "/v2/")
    .with_status(200)
    .with_header(API_VERSION_K, API_VERSION_V)
    .create();

  let client = docker_registry::v2::Client::configure()
    .registry(&addr)
    .insecure_registry(true)
    .username(None)
    .password(None)
    .build()
    .unwrap();

  let ok = client.is_v2_supported().await.unwrap();

  mock.assert_async().await;
  assert!(ok);

  let _ensure_v2 = client.ensure_v2_registry().await.unwrap();
}

#[tokio::test]
async fn test_version_check_status_no_auth() {
  let mut server = mockito::Server::new_async().await;
  let addr = server.host_with_port();

  let mock = server
    .mock("GET", "/v2/")
    .with_status(401)
    .with_header(API_VERSION_K, API_VERSION_V)
    .create();

  let client = docker_registry::v2::Client::configure()
    .registry(&addr)
    .insecure_registry(true)
    .username(None)
    .password(None)
    .build()
    .unwrap();

  let res = client.is_v2_supported().await.unwrap();

  mock.assert_async().await;
  assert!(res);
}

#[tokio::test]
async fn test_version_check_status_not_found() {
  let mut server = mockito::Server::new_async().await;
  let addr = server.host_with_port();

  let mock = server
    .mock("GET", "/v2/")
    .with_status(404)
    .with_header(API_VERSION_K, API_VERSION_V)
    .create();

  let client = docker_registry::v2::Client::configure()
    .registry(&addr)
    .insecure_registry(true)
    .username(None)
    .password(None)
    .build()
    .unwrap();

  let res = client.is_v2_supported().await.unwrap();

  mock.assert_async().await;
  assert!(!res);
}

#[tokio::test]
async fn test_version_check_status_forbidden() {
  let mut server = mockito::Server::new_async().await;
  let addr = server.host_with_port();

  let mock = server
    .mock("GET", "/v2/")
    .with_status(403)
    .with_header(API_VERSION_K, API_VERSION_V)
    .create();

  let client = docker_registry::v2::Client::configure()
    .registry(&addr)
    .insecure_registry(true)
    .username(None)
    .password(None)
    .build()
    .unwrap();

  let res = client.is_v2_supported().await.unwrap();

  mock.assert_async().await;
  assert!(!res);
}

#[tokio::test]
async fn test_version_check_no_header() {
  let mut server = mockito::Server::new_async().await;
  let addr = server.host_with_port();
  let mock = server.mock("GET", "/v2/").with_status(403).create_async().await;

  let client = docker_registry::v2::Client::configure()
    .registry(&addr)
    .insecure_registry(true)
    .username(None)
    .password(None)
    .build()
    .unwrap();

  let res = client.is_v2_supported().await.unwrap();

  mock.assert_async().await;
  assert!(!res);
}

#[tokio::test]
async fn test_version_check_trailing_slash() {
  let mut server = mockito::Server::new_async().await;
  let addr = server.host_with_port();

  let _mock = server
    .mock("GET", "/v2")
    .with_status(200)
    .with_header(API_VERSION_K, API_VERSION_V)
    .create();

  let client = docker_registry::v2::Client::configure()
    .registry(&addr)
    .insecure_registry(true)
    .username(None)
    .password(None)
    .build()
    .unwrap();

  let res = client.is_v2_supported().await.unwrap();

  // TODO - why does this fail?
  // mock.assert_async().await;
  assert!(!res);
}
