// Copyright 2020 Contributors to the Parsec project.
// SPDX-License-Identifier: Apache-2.0
use crate::constants::algorithm::HashingAlgorithm;
use crate::structures::Digest;
use crate::tss2_esys::TPML_DIGEST_VALUES;
use crate::{Error, Result};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone, Default)]
pub struct DigestValues {
    digests: HashMap<HashingAlgorithm, Digest>,
}

impl DigestValues {
    pub fn new() -> Self {
        DigestValues {
            digests: HashMap::new(),
        }
    }

    pub fn set(&mut self, alg: HashingAlgorithm, dig: Digest) {
        let _ = self.digests.insert(alg, dig);
    }
}

impl TryFrom<DigestValues> for TPML_DIGEST_VALUES {
    type Error = Error;
    fn try_from(digest_values: DigestValues) -> Result<Self> {
        let mut digest_values = digest_values;
        let mut tss_digest_values: TPML_DIGEST_VALUES = Default::default();
        for (digest_hash, digest_val) in digest_values.digests.drain() {
            tss_digest_values.digests[tss_digest_values.count as usize] =
                (digest_hash, digest_val).try_into()?;
            tss_digest_values.count += 1;
        }
        Ok(tss_digest_values)
    }
}
