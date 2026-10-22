//! The original's random sources: `arc4random()` (most gameplay) and `random()`/`rand()`
//! (seeded once with `srandom(time(0))` in `BCView::initWithFrame:`).

use bevy_ecs::prelude::*;

/// A PCG32 generator standing in for the C library generators.
#[derive(Resource)]
pub struct Random {
    state: u64,
    increment: u64,
}

impl Random {
    pub fn with_seed(seed: u64) -> Self {
        let mut random = Self {
            state: 0,
            increment: (seed << 1) | 1,
        };
        random.next_u32();
        random.state = random.state.wrapping_add(seed);
        random.next_u32();
        random
    }

    fn next_u32(&mut self) -> u32 {
        let old_state = self.state;
        self.state = old_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(self.increment);
        let xorshifted = (((old_state >> 18) ^ old_state) >> 27) as u32;
        let rotation = (old_state >> 59) as u32;
        xorshifted.rotate_right(rotation)
    }

    /// `arc4random()`: uniform `u32`.
    pub fn arc4random(&mut self) -> u32 {
        self.next_u32()
    }

    /// `random()` / `rand()`: uniform in `0..=0x7fff_ffff`.
    pub fn random(&mut self) -> i32 {
        let value = self.next_u32() >> 1;
        value as i32
    }

    /// `arc4random() % n` with the original's u32 semantics (`n == 0` returns the raw value).
    pub fn arc4random_mod(&mut self, n: u32) -> u32 {
        let value = self.arc4random();
        if n == 0 { value } else { value % n }
    }
}

/// A fixed seed; the platform layer reseeds with the clock at start-up (like
/// `srandom(time(0))`), which keeps this crate free of OS time APIs (unavailable on wasm).
impl Default for Random {
    fn default() -> Self {
        Self::with_seed(0x5eed_d00d)
    }
}
