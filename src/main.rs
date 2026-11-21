use std::sync::Arc;

use kallbet::config::Config;
use kallbet::state::AppState;
use kallbet::{app, db, runtime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // `.env` is for local development only; real env vars win over it.
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env()?;
    tracing::info!(?config, "starting KallBet");
    if config.dev_login {
        tracing::warn!("KALLBET_DEV_LOGIN is on: anyone can sign in as anyone. Never use in production.");
    }
    if !config.web_dir.join("index.html").is_file() {
        tracing::warn!(
            "no built SPA at {}: run `npm run build` in web/",
            config.web_dir.display()
        );
    }

    let db = db::connect(&config.database_url).await?;
    let clock = runtime::clock();
    let state = AppState {
        source: runtime::match_source(&config, &db, clock.clone()),
        db,
        clock,
        notifier: runtime::notifier(&config),
        config: Arc::new(config),
    };
    runtime::spawn_background_tasks(&state);

    let listener = tokio::net::TcpListener::bind(state.config.bind_addr).await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app::build_router(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
