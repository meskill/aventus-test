use std::sync::Arc;

use tokio::net::{TcpListener, ToSocketAddrs};

use axum::{extract::State, routing::post, Json, Router};
use tokio::join;

struct AppState {
    name: String,
}

async fn start_server(app: Router, addr: impl ToSocketAddrs) {
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[tokio::main]
async fn main() {
    let app = Router::new().route(
        "/updatePrice",
        post(|State(state): State<Arc<AppState>>, Json(payload): Json<serde_json::Value>| async move {
            println!("Got API request for {} API:\n{}\n", state.name, payload);
            "Acquired!"
        }),
    );

    join!(
        start_server(
            app.clone().with_state(Arc::new(AppState {
                name: "Gold".to_string()
            })),
            "0.0.0.0:3000"
        ),
        start_server(
            app.clone().with_state(Arc::new(AppState {
                name: "Silver".to_string()
            })),
            "0.0.0.0:3001"
        )
    );
}
