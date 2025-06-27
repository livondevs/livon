use lazy_static::lazy_static;
use rand::{rng, rngs::StdRng, RngCore, SeedableRng};
use std::{env, sync::Mutex};

lazy_static! {
    /// Global RAND ID generator protected by a mutex.
    pub static ref RAND_ID_GENERATOR: Mutex<RandIdGenerator> = Mutex::new(RandIdGenerator::new());
}

pub struct RandIdGenerator {
    /// A single-byte seed for deterministic generation in test mode.
    seed: u8,
}

impl RandIdGenerator {
    /// Create a new generator, seed starts at zero.
    pub fn new() -> Self {
        Self { seed: 0 }
    }

    /// Generate a purely random ID of length 21 (no seeding).
    pub fn gen_random(&self) -> String {
        const DEFAULT_LEN: usize = 21;
        let mut rng = rng();
        Self::fill_id_with(&mut rng, DEFAULT_LEN)
    }

    /// Generate a seeded (deterministic) ID of length 21.
    /// The internal seed is incremented on each call.
    pub fn gen_seeded(&mut self) -> String {
        const DEFAULT_LEN: usize = 21;
        // Expand the single-byte seed to a 32-byte array.
        let seed_bytes = [self.seed; 32];
        self.seed = self.seed.wrapping_add(1);
        let mut rng = StdRng::from_seed(seed_bytes);
        Self::fill_id_with(&mut rng, DEFAULT_LEN)
    }

    /// Choose between seeded or random based on the `LUNAS_TEST` env var.
    pub fn gen(&mut self) -> String {
        if Self::is_testgen() {
            self.gen_seeded()
        } else {
            self.gen_random()
        }
    }

    pub fn reset(&mut self) {
        self.seed = 0;
    }

    /// Internal helper: fill a string of given length using the provided RNG.
    fn fill_id_with<R: RngCore>(rng: &mut R, len: usize) -> String {
        // Allowed characters for the ID.
        const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz\
                                   ABCDEFGHIJKLMNOPQRSTUVWXYZ$";

        (0..len)
            .map(|_| {
                let idx = (rng.next_u64() as usize) % ALPHABET.len();
                ALPHABET[idx] as char
            })
            .collect()
    }

    /// Test mode is active if the environment variable `LUNAS_TEST` is set.
    fn is_testgen() -> bool {
        env::var("LUNAS_TEST").is_ok()
    }
}
