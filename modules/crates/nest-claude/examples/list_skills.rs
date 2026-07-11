//! Live API smoke test: lists Skills on your Anthropic account, then fetches
//! the first one by id to verify `list_skills`/`get_skill` round-trip.
//!
//! Requires `ANTHROPIC_API_KEY` in the environment. Run with:
//!
//! ```bash
//! cargo run --example list_skills -p nest-claude
//! ```

use nest_claude::{ClaudeClient, ClaudeConfig, ListSkillsParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClaudeConfig::from_env()?;
    let client = ClaudeClient::new(config)?;

    let page = client
        .list_skills(ListSkillsParams::new().limit(20))
        .await?;

    println!(
        "{} skill(s) (has_more: {}):",
        page.data.len(),
        page.has_more
    );
    for skill in &page.data {
        println!(
            "  {} — {:?} — {} (v{})",
            skill.id, skill.source, skill.display_title, skill.latest_version
        );
    }

    if let Some(first) = page.data.first() {
        let fetched = client.get_skill(&first.id).await?;
        println!("\nGet Skill round-trip for {}:", fetched.id);
        println!("  display_title:  {}", fetched.display_title);
        println!("  updated_at:     {}", fetched.updated_at);
    }

    Ok(())
}
