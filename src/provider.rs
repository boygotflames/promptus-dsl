use anyhow::Result;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum Provider {
    #[default]
    Generic,
    Openai,
    Anthropic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportStatus {
    Supported,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowProfile {
    /// Compact `@`-marker format (generic and openai providers).
    V0,
    /// XML-tag format optimized for Anthropic/Claude (anthropic provider).
    V1Anthropic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenizerProfile {
    Cl100kBase,
    O200kBase,
}

impl TokenizerProfile {
    pub fn identifier(self) -> &'static str {
        match self {
            TokenizerProfile::Cl100kBase => "cl100k_base",
            TokenizerProfile::O200kBase => "o200k_base",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderProfile {
    provider: Provider,
}

impl Provider {
    pub fn profile(self) -> ProviderProfile {
        ProviderProfile { provider: self }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Provider::Generic => "generic",
            Provider::Openai => "openai",
            Provider::Anthropic => "anthropic",
        }
    }
}

impl ProviderProfile {
    pub fn provider(self) -> Provider {
        self.provider
    }

    pub fn shadow_support(self) -> SupportStatus {
        match self.provider {
            Provider::Generic | Provider::Openai | Provider::Anthropic => SupportStatus::Supported,
        }
    }

    pub fn tokenizer_support(self) -> SupportStatus {
        match self.provider {
            Provider::Generic | Provider::Openai | Provider::Anthropic => SupportStatus::Supported,
        }
    }

    pub fn shadow_profile(self) -> Result<ShadowProfile> {
        match self.provider {
            Provider::Generic | Provider::Openai => Ok(ShadowProfile::V0),
            Provider::Anthropic => Ok(ShadowProfile::V1Anthropic),
        }
    }

    pub fn tokenizer_profile(self) -> Result<TokenizerProfile> {
        match self.provider {
            Provider::Generic | Provider::Openai => Ok(TokenizerProfile::Cl100kBase),
            Provider::Anthropic => Ok(TokenizerProfile::O200kBase),
        }
    }
}
