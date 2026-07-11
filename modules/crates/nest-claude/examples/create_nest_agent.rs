//! Live API smoke test: creates a "Nest Framework Agent" wired to the
//! previously-created "nest-core" custom Skill, via `POST /v1/agents`.
//!
//! Requires `ANTHROPIC_API_KEY` in the environment, and requires the
//! "nest-core" skill to already exist (run the `create_nest_core_skill`
//! example first). Run with:
//!
//! ```bash
//! cargo run --example create_nest_agent -p nest-claude
//! ```
//!
//! This is a one-time setup step, not something to re-run per session — see
//! `docs/plan/nest-claude-v1.md` § Live resources for the agent id this
//! produces. More skills will be attached to the same agent over time via
//! `update_agent`, as more Nest crates get their own Skill.

use nest_claude::{
    AgentSkillRef, AgentTool, ClaudeClient, ClaudeConfig, CreateAgentRequest, ListSkillsParams,
    SkillSource,
};

const AGENT_NAME: &str = "Nest Framework Agent";
const AGENT_SYSTEM_PROMPT: &str = "You are a coding agent for the Nest framework (a modular Rust \
application framework at github.com/pacificnm/nest). Consult your attached Skills for \
authoritative, verified documentation on individual Nest crates before writing or reviewing code \
that touches them — prefer a Skill's guidance over your own prior knowledge of the crate, since \
each Skill is written and refreshed from the crate's actual source, not general Rust conventions. \
More Nest crate Skills will be attached to you over time as they are documented; use whichever \
are relevant to the task at hand.";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClaudeConfig::from_env()?;
    let client = ClaudeClient::new(config)?;

    // Look the skill up by name rather than hardcoding its id, so this example
    // stays correct if it's ever re-run against a different account.
    let page = client
        .list_skills(
            ListSkillsParams::new()
                .limit(100)
                .source(SkillSource::Custom),
        )
        .await?;
    let nest_core_skill = page
        .data
        .into_iter()
        .find(|skill| skill.display_title == "nest-core")
        .ok_or("no \"nest-core\" skill found — run the create_nest_core_skill example first")?;

    println!("Found nest-core skill: {}", nest_core_skill.id);

    let agent = client
        .create_agent(
            CreateAgentRequest::new(AGENT_NAME, "claude-opus-4-8")
                .description("Coding agent for the Nest Rust framework, with one Skill per documented crate.")
                .system(AGENT_SYSTEM_PROMPT)
                .tools(vec![AgentTool::agent_toolset()])
                .skills(vec![AgentSkillRef::custom(nest_core_skill.id)]),
        )
        .await?;

    println!("\nCreated agent:");
    println!("  id:      {}", agent.id);
    println!("  version: {}", agent.version);
    println!("  name:    {}", agent.name);
    println!("  skills:  {:?}", agent.skills);
    println!(
        "\nStore this agent id and reuse it — creating an agent is a one-time setup step, not \
         something to repeat per run. To attach another skill later:\n  \
         update_agent(&agent.id, UpdateAgentRequest::new(agent.version).skills(vec![...]))"
    );

    Ok(())
}
