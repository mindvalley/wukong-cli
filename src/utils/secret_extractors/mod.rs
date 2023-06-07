use std::path::{Path, PathBuf};

mod elixir_config;
mod wk_toml_config;

pub use elixir_config::ElixirConfigExtractor;
pub use wk_toml_config::WKTomlConfigExtractor;

//
//  ┌────────────────────────────────────────────────────┐                   ┌──────────────────────┐
//  │                                                    │                   │                      │
//  │                            SecretExtractor         │                   │                      │
//  │                    ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─   │                   │                      │
//  │                                                 │  │                   │  ┌────────────────┐  │
//  │                    │                               │  Vec<SecretInfo>  │  │                │  │
//  │ ┌──────────────┐      ┌───────────────────────┐ │  │         ┌─────────┼─▶│  SecretPusher  │  │
//  │ │              │   │  │                       │    │         │         │  │                │  │
//  │ │ .wukong.toml │──────▶ WKTomlConfigExtractor │ │  │         │         │  └────────────────┘  │
//  │ │              │   │  │                       │    │         │         │                      │
//  │ └──────────────┘      └───────────────────────┘ │  │         │         │  ┌────────────────┐  │
//  │                    │                               │  Vec<SecretInfo>  │  │                │  │
//  │                                                 │──┼─────────┼─────────┼─▶│  SecretPuller  │  │
//  │                    │                               │         │         │  │                │  │
//  │ ┌──────────────┐      ┌───────────────────────┐ │  │         │         │  └────────────────┘  │
//  │ │              │   │  │                       │    │         │         │                      │
//  │ │   dev.exs    │──────▶ ElixirConfigExtractor │ │  │         │         │  ┌────────────────┐  │
//  │ │              │   │  │                       │    │  Vec<SecretInfo>  │  │                │  │
//  │ └──────────────┘      └───────────────────────┘ │  │         └─────────┼─▶│  SecretDiffer  │  │
//  │                    │                               │                   │  │                │  │
//  │                     ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘  │                   │  └────────────────┘  │
//  │                                                    │                   │                      │
//  └────────────────────────────────────────────────────┘                   └──────────────────────┘
//

pub struct SecretInfo {
    pub key: String,
    // Provider, such as "bunker"
    pub provider: String,
    // Kind, such as "elixir_config" or "generic"
    pub kind: String,
    // the secret source path on the bunker if the provider is "bunker"
    pub src: String,
    // the destination file path on the local
    pub destination_file: String,
    // the secret name/key on the bunker
    pub name: String,
    pub annotated_file: PathBuf,
}

pub trait SecretExtractor {
    fn extract(file: &Path) -> Vec<SecretInfo>;
}
