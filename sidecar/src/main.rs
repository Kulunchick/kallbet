use kallbet_ai::*;
#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load(std::env::var("SIDECAR_CONFIG")?)?;
    let mut sidecar = Sidecar::from_config(config, std::env::var("KALLBET_BOT_SECRET")?).await?;
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        if let Err(e) = sidecar.tick().await {
            eprintln!("Sidecar tick failed: {e}");
        }
    }
}
