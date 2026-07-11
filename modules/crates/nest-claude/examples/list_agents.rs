//! Live API smoke test: lists Agents on your Anthropic account, then fetches
//! the first one by id to verify `list_agents`/`get_agent` round-trip.
//!
//! Requires `ANTHROPIC_API_KEY` in the environment. Run with:
//!
//! ```bash
//! cargo run --example list_agents -p nest-claude
//! ```

use nest_claude::{ClaudeClient, ClaudeConfig, ListAgentsParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClaudeConfig::from_env()?;
    let client = ClaudeClient::new(config)?;

    let page = client
        .list_agents(ListAgentsParams::new().include_archived(false))
        .await?;

    println!("{} agent(s):", page.data.len());
    for agent in &page.data {
        println!(
            "  {} — {} (v{}, model {})",
            agent.id, agent.name, agent.version, agent.model.id
        );
    }

    if let Some(first) = page.data.first() {
        let fetched = client.get_agent(&first.id, None).await?;
        println!("\nGet Agent round-trip for {}:", fetched.id);
        println!("  system:  {:?}", fetched.system);
        println!("  tools:   {:?}", fetched.tools);
        println!("  skills:  {:?}", fetched.skills);
        println!("  updated: {}", fetched.updated_at);
    }

    Ok(())
}
