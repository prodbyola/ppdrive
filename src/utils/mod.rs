pub mod fs;
pub mod jwt;
pub mod sqlx;
pub mod tools;

use std::path::PathBuf;

use tools::{
    client::{create_client, regenerate_token},
    secrets::generate_secret,
};

use crate::{
    config::{configor::run_configor, AppConfig},
    errors::AppError,
    state::AppState,
};

pub fn get_env(key: &str) -> Result<String, AppError> {
    std::env::var(key).map_err(|err| {
        tracing::error!("unable to get var {key}: {err}");
        err.into()
    })
}

pub fn mb_to_bytes(value: usize) -> usize {
    value * 1024 * 1000
}

pub fn install_dir() -> Result<PathBuf, AppError> {
    let exec_path = std::env::current_exe()?;
    let path = exec_path
        .parent()
        .ok_or(AppError::IOError("unable to get install dir".to_string()))?;

    Ok(path.to_owned())
}

/// if specified, run ppdrive extra args
pub async fn args_runner(args: Vec<String>, config: &AppConfig) -> Result<(), AppError> {
    if let Some(a1) = args.get(1) {
        let a1 = &a1.as_str();
        if ["--version", "-v"].contains(a1) {
            let n = std::env::var("CARGO_PKG_NAME")?;
            let v = std::env::var("CARGO_PKG_VERSION")?;

            println!("{n}: {v}");
        } else if ["--create_client", "--token"].contains(a1) {
            let is_new = a1 == &"--create_client";

            match args.get(2) {
                Some(spec) => {
                    let state = AppState::new(config).await?;
                    let token = if is_new {
                        create_client(&state, spec).await?
                    } else {
                        regenerate_token(&state, spec).await?
                    };

                    println!("CLIENT_TOKEN: {token}");
                }
                None => {
                    let spec = if is_new { "name" } else { "id" };
                    panic!("client creation failed: please specify client {spec}.");
                }
            }
        } else if *a1 == "xgen" {
            generate_secret().await?;
            tracing::info!("secret keys generated and saved!");
        } else if ["--configure", "-c"].contains(a1) {
            run_configor().await?;
        } else {
            panic!("unknown command {}", a1);
        }
    }

    Ok(())
}
