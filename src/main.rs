use std::{path::Path, sync::Arc};

use axum::{extract::State, http::Method, response::IntoResponse, routing::get, Json, Router};

use config::Config;
use google_sheets4::{
    hyper::{client::HttpConnector, Client},
    hyper_rustls::{HttpsConnector, HttpsConnectorBuilder},
    oauth2::{authenticator::Authenticator, read_service_account_key, ServiceAccountAuthenticator},
    Sheets,
};
use rand::{seq::SliceRandom, thread_rng};
use serde_json::Value;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod config;

struct RouterState {
    hub: Sheets<HttpsConnector<HttpConnector>>,
    config: Config,
}

async fn fetch_data(State(state): State<Arc<RouterState>>) -> impl IntoResponse {
    let (_, a) = state
        .hub
        .spreadsheets()
        .values_get(
            state.config.get_sheet_id().as_str(),
            format!(
                "{}!{}",
                state.config.get_sheet_name(),
                state.config.get_range(),
            )
            .as_str(),
        )
        .doit()
        .await
        .unwrap();

    let mut rng = thread_rng();

    let mut data = a
        .values
        .unwrap()
        .into_iter()
        .flatten()
        .collect::<Vec<Value>>();

    data.shuffle(&mut rng);

    Json(data)
}

#[tokio::main]
async fn main() {
    let config = Config::new(Path::new("Config.toml")).await.unwrap();

    let client: Client<HttpsConnector<HttpConnector>> = Client::builder().build(
        HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .enable_http2()
            .build(),
    );

    let au = auth(&client, Path::new(config.get_auth_file())).await;

    let hub = Sheets::new(client, au);

    let state = Arc::new(RouterState { hub, config });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(Any);

    let router = Router::new()
        .route("/fetch", get(fetch_data))
        .with_state(state)
        .layer(cors);

    let listener = TcpListener::bind("0.0.0.0:1327").await.unwrap();

    axum::serve(listener, router).await.unwrap();
}

async fn auth(
    client: &Client<HttpsConnector<HttpConnector>>,
    auth_path: &Path,
) -> Authenticator<HttpsConnector<HttpConnector>> {
    let secret = read_service_account_key(auth_path).await.unwrap();

    ServiceAccountAuthenticator::with_client(secret, client.clone())
        .build()
        .await
        .unwrap()
}
