use axum::{
    extract::{Multipart, Path, State},
    routing::{post, get},
    Router,
};
use axum_macros::debug_handler;
use tokio::{fs::File, io::AsyncWriteExt};
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        asset::{Asset, CreateAssetOptions},
        AssetType,
    },
    state::AppState,
};

use super::extractors::UserExtractor;

#[debug_handler]
async fn get_asset(
    Path(asset_path): Path<String>,
    State(state): State<AppState>,
) -> Result<String, AppError> {
    let pool = state.pool().await;
    let mut conn = pool.get().await?;

    let asset = Asset::get_by_path(&mut conn, asset_path).await?;

    if asset.public {
        // TODO: build and return asset object
        Ok(asset.asset_path)
    } else {
        Err(AppError::InternalServerError("access denied".to_string()))
    }
}

#[debug_handler]
async fn create_asset(
    State(state): State<AppState>,
    UserExtractor(current_user): UserExtractor,
    mut multipart: Multipart,
) -> Result<String, AppError> {
    if current_user.can_create() {
        let mut opts = CreateAssetOptions {
            user: current_user.id.clone(),
            ..Default::default()
        };

        while let Some(mut field) = multipart.next_field().await? {
            let name = field.name().unwrap_or("").to_string();

            if name == "path" {
                opts.path = field.text().await?;
            } else if name == "public" {
                let public = field.text().await?;
                opts.public = matches!(public.as_str(), "true" | "1" | "yes")
            } else if name == "asset_type" {
                opts.asset_type = AssetType::try_from(name.as_str())?;
            } else if name == "create_parents" {
                let create_parents = field.text().await?;
                opts.create_parents = matches!(create_parents.as_str(), "true" | "1" | "yes");
            } else if name == "file" {
                // TODO: Extract file extension/mime-type
                // let filename = field.file_name().map(|s| s.to_string());

                let tmp_name = Uuid::new_v4().to_string();
                let tmp_path = format!("./tmp/{tmp_name}");
                let mut file = File::create(&tmp_path).await?;

                while let Some(chunk) = field.chunk().await? {
                    file.write_all(&chunk).await?;
                }

                opts.tmp_file = Some(tmp_path);
            }
        }

        let pool = state.pool().await;
        let mut conn = pool.get().await?;

        let path = Asset::create(&mut conn, opts).await?;
        Ok(path)
    } else {
        Err(AppError::AuthorizationError(
            "permission denied".to_string(),
        ))
    }
}

pub fn asset_routes() -> Router<AppState> {
    Router::new()
    .route("/:asset_path", get(get_asset))
    .route("/create", post(create_asset))
}
