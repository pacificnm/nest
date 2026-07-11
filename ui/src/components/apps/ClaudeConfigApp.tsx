/**
 * Claude Config - Anthropic API key, Skills, and Agents viewer for Nest
 * Desktop. Same layout as Settings: left menu of categories, main panel
 * shows the selected category. Distinct from the "Nest Agent" app (a local
 * PTY-launched coding agent) — this one talks to api.anthropic.com directly
 * via `nest-claude`, to manage persisted Managed Agents + Skills.
 */

import { useCallback, useEffect, useState } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faKey, faRobot, faWandMagicSparkles } from "@fortawesome/free-solid-svg-icons";
import { claudeListAgents, claudeListSkills, type Agent, type Skill } from "../../lib/claude";
import { ApiKeyPanel } from "./claude/ApiKeyPanel";
import { SkillsPanel } from "./claude/SkillsPanel";
import { AgentsPanel } from "./claude/AgentsPanel";

type Category = "apiKey" | "skills" | "agents";

const CATEGORIES: { id: Category; label: string; icon: typeof faKey }[] = [
  { id: "apiKey", label: "API Key", icon: faKey },
  { id: "skills", label: "Skills", icon: faWandMagicSparkles },
  { id: "agents", label: "Agents", icon: faRobot },
];

export function ClaudeConfigApp() {
  const [selected, setSelected] = useState<Category>("agents");

  const [skills, setSkills] = useState<Skill[]>([]);
  const [skillsLoading, setSkillsLoading] = useState(true);
  const [skillsError, setSkillsError] = useState<string | null>(null);

  const [agents, setAgents] = useState<Agent[]>([]);
  const [agentsLoading, setAgentsLoading] = useState(true);
  const [agentsError, setAgentsError] = useState<string | null>(null);

  const loadSkills = useCallback(() => {
    setSkillsLoading(true);
    setSkillsError(null);
    claudeListSkills()
      .then(setSkills)
      .catch((error) => setSkillsError(error instanceof Error ? error.message : String(error)))
      .finally(() => setSkillsLoading(false));
  }, []);

  const loadAgents = useCallback(() => {
    setAgentsLoading(true);
    setAgentsError(null);
    claudeListAgents()
      .then(setAgents)
      .catch((error) => setAgentsError(error instanceof Error ? error.message : String(error)))
      .finally(() => setAgentsLoading(false));
  }, []);

  useEffect(() => {
    loadSkills();
    loadAgents();
  }, [loadSkills, loadAgents]);

  return (
    <div className="settings-app">
      <aside className="settings-app-sidebar">
        <div className="settings-app-sidebar-header">
          <h2 className="text-sm font-semibold text-nest-foreground">Claude Config</h2>
          <p className="text-xs text-nest-muted">API key, Skills, Agents</p>
        </div>
        <nav className="settings-app-toc">
          {CATEGORIES.map((category) => (
            <button
              key={category.id}
              type="button"
              className={["settings-app-toc-item", selected === category.id ? "settings-app-toc-item-active" : ""].join(" ")}
              onClick={() => setSelected(category.id)}
            >
              <FontAwesomeIcon icon={category.icon} className="size-3.5" />
              {category.label}
            </button>
          ))}
        </nav>
      </aside>
      <main className="settings-app-content">
        {selected === "apiKey" ? <ApiKeyPanel /> : null}
        {selected === "skills" ? (
          <SkillsPanel skills={skills} loading={skillsLoading} error={skillsError} onRefresh={loadSkills} />
        ) : null}
        {selected === "agents" ? (
          <AgentsPanel agents={agents} skills={skills} loading={agentsLoading} error={agentsError} onRefresh={loadAgents} />
        ) : null}
      </main>
    </div>
  );
}
