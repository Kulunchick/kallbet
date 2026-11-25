use crate::*;
use serde::{Deserialize, Serialize};

#[async_trait::async_trait]
pub trait PickemStep: Send + Sync {
    async fn tick(
        &self,
        platform: &dyn LeaguePlatform,
        token: &str,
        bot: &BotProfile,
        league: i64,
        now: Instant,
        audit: &Audit,
    ) -> Result<()>;
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PairwiseProbability {
    pub first: i64,
    pub second: i64,
    pub map: f64,
}

/// Injectable uniform random source for Monte-Carlo Stage simulation.
pub trait RandomSource: Send {
    fn next_u64(&mut self) -> u64;

    fn next_f64(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) * (1.0 / ((1u64 << 53) as f64))
    }
}

/// Deterministic Xoshiro256++ PRNG initialised from a 64-bit seed via SplitMix64.
#[derive(Clone, Debug)]
pub struct SeededRng {
    state: [u64; 4],
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        let mut z = seed;
        let mut next_splitmix = || {
            z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut x = z;
            x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            x ^ (x >> 31)
        };
        Self {
            state: [
                next_splitmix(),
                next_splitmix(),
                next_splitmix(),
                next_splitmix(),
            ],
        }
    }
}

impl RandomSource for SeededRng {
    fn next_u64(&mut self) -> u64 {
        let result = self.state[0]
            .wrapping_add(self.state[3])
            .rotate_left(23)
            .wrapping_add(self.state[0]);
        let t = self.state[1] << 17;
        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);
        result
    }
}

/// #28 implements the deterministic solver and its League-level scheduling adapter.
pub trait PickemSolver: Send + Sync {
    fn solve(
        &self,
        stage: &AsOf<serde_json::Value>,
        ratings: &[AsOf<PairwiseProbability>],
        cutoff: Instant,
    ) -> Result<Option<serde_json::Value>>;
}
