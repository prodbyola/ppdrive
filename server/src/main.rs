use crate::app::initialize_app;
use errors::AppError;
use ppdrive_core::config::{get_config_path, AppConfig};

mod app;
mod errors;
mod routes;
mod state;
mod utils;

type AppResult<T> = Result<T, AppError>;

#[tokio::main]
async fn main() -> AppResult<()> {
    let config_path = get_config_path()?;
    let config = AppConfig::load(config_path).await?;
    let app = initialize_app(&config).await?;

    match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.server().port())).await {
        Ok(listener) => {
            if let Ok(addr) = listener.local_addr() {
                tracing::info!("listening on {addr}");
            }

            axum::serve(listener, app)
                .await
                .map_err(|err| AppError::InitError(err.to_string()))?;
        }
        Err(err) => {
            tracing::error!("Error starting listener: {err}");
            panic!("{err:?}")
        }
    }

    Ok(())
}
