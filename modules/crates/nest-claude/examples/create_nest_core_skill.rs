//! Live API smoke test: creates a real "nest-core" Skill on your Anthropic
//! account via `POST /v1/skills`.
//!
//! Requires `ANTHROPIC_API_KEY` in the environment. Run with:
//!
//! ```bash
//! cargo run --example create_nest_core_skill -p nest-claude
//! ```

use nest_claude::{ClaudeClient, ClaudeConfig, FileUpload};

const SKILL_MD: &str = include_str!("nest-core-skill/SKILL.md");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClaudeConfig::from_env()?;
    let client = ClaudeClient::new(config)?;

    let skill = client
        .create_skill(vec![FileUpload::new(
            "nest-core/SKILL.md",
            SKILL_MD.as_bytes().to_vec(),
        )])
        .await?;

    println!("Created skill:");
    println!("  id:             {}", skill.id);
    println!("  display_title:  {}", skill.display_title);
    println!("  source:         {:?}", skill.source);
    println!("  latest_version: {}", skill.latest_version);
    println!("  created_at:     {}", skill.created_at);

    Ok(())
}
