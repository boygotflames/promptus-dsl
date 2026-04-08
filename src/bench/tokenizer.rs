use anyhow::{Context, Result};
use tiktoken_rs::{CoreBPE, cl100k_base};

use crate::provider::{Provider, TokenizerProfile};

pub const DEFAULT_ENCODING_NAME: &str = "cl100k_base";

pub struct TokenCounter {
    encoder: CoreBPE,
    profile: TokenizerProfile,
}

impl TokenCounter {
    pub fn new() -> Result<Self> {
        Self::for_provider(Provider::Generic)
    }

    pub fn for_provider(provider: Provider) -> Result<Self> {
        let profile = provider.profile().tokenizer_profile()?;
        Self::from_profile(profile)
    }

    fn from_profile(profile: TokenizerProfile) -> Result<Self> {
        let encoder = cl100k_base().context(format!(
            "failed to initialize tokenizer encoding {DEFAULT_ENCODING_NAME}"
        ))?;

        Ok(Self { encoder, profile })
    }

    pub fn encoding_name(&self) -> &'static str {
        self.profile.identifier()
    }

    pub fn count(&self, input: &str) -> usize {
        self.encoder.encode_with_special_tokens(input).len()
    }
}

pub fn count_tokens(input: &str) -> Result<usize> {
    let counter = TokenCounter::new()?;
    Ok(counter.count(input))
}
