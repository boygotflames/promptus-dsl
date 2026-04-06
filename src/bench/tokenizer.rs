use anyhow::Result;
use tiktoken_rs::cl100k_base;

pub fn count_tokens(input: &str) -> Result<usize> {
    let encoder = cl100k_base()?;
    Ok(encoder.encode_with_special_tokens(input).len())
}
