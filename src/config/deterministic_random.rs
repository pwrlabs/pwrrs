use sha2::{Sha256, Digest};
use std::io::{self, Read};

pub struct DeterministicSecureRandom {
    digest: Sha256,
    seed: Vec<u8>,
    counter: u32,
}

impl DeterministicSecureRandom {
    pub fn new(seed: &[u8]) -> Self {
        Self {
            digest: Sha256::new(),
            seed: seed.to_vec(),
            counter: 0,
        }
    }

    pub fn next_bytes(&mut self, bytes: &mut [u8]) {
        let mut index = 0;
        while index < bytes.len() {
            let mut digest = self.digest.clone();
            digest.update(&self.seed);
            digest.update(&self.counter.to_be_bytes());
            let hash = digest.finalize();

            let to_copy = std::cmp::min(hash.len(), bytes.len() - index);
            bytes[index..index + to_copy].copy_from_slice(&hash[..to_copy]);
            index += to_copy;
            self.counter += 1;
        }
    }
}

impl Read for DeterministicSecureRandom {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.next_bytes(buf);
        Ok(buf.len())
    }
}
