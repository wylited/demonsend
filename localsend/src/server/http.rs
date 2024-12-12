use axum::{
    routing::{get, post}, Extension, Json, Router
};
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::{discovery::http::register_device, Client};

impl Client {
    pub async fn start_http_server(&self) -> crate::error::Result<()> {
        let app = self.create_router();
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));

        let listener = TcpListener::bind(&addr).await?;
        println!("HTTP server listening on {}", addr);

        axum::serve(listener, app).await?;
        Ok(())
    }

    fn create_router(&self) -> Router {
        let peers = self.peers.clone();
        let device = self.device.clone();

        Router::new()
            .route("/api/localsend/v2/register", post(register_device))
            .route("/api/localsend/v2/info", get(move || {
                let device = device.clone();
                async move { Json(device) }
            }))
            .layer(Extension(self.device.clone()))
            .with_state(peers)

    }
}
