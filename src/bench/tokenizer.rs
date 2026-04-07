use anyhow::{Context, Result};
use tiktoken_rs::{CoreBPE, cl100k_base};

pub const DEFAULT_ENCODING_NAME: &str = "cl100k_base";

pub struct TokenCounter {
    encoder: CoreBPE,
}

impl TokenCounter {
    pub fn new() -> Result<Self> {
        let encoder = cl100k_base().context(format!(
            "failed to initialize tokenizer encoding {DEFAULT_ENCODING_NAME}"
        ))?;

        Ok(Self { encoder })
    }

    pub fn encoding_name(&self) -> &'static str {
        DEFAULT_ENCODING_NAME
    }

    pub fn count(&self, input: &str) -> usize {
        self.encoder.encode_with_special_tokens(input).len()
    }
}

pub fn count_tokens(input: &str) -> Result<usize> {
    let counter = TokenCounter::new()?;
    Ok(counter.count(input))
}
