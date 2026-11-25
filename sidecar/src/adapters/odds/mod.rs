//! Pre-match series probabilities in OddsPapi, Polymarket, Kalshi priority order.
use crate::{Audit, OddsSource, Result};
use std::sync::Arc;
mod chain;
mod feed;
pub mod kalshi;
mod lines;
pub mod oddspapi;
pub mod polymarket;

pub use chain::{FallbackOdds, OddsConfig};
pub use feed::{FetchError, OddsTransport};

pub fn build(audit: &Audit) -> Result<Arc<dyn OddsSource>> {
    Ok(Arc::new(FallbackOdds::new(
        audit,
        OddsConfig::from_env()?,
        Arc::new(feed::HttpTransport::new()?),
    )))
}
