/**
 * Claude Config — Agents category. Read-only list of every non-archived
 * Agent on the account, with each agent's assigned skills resolved to their
 * display titles (via the Skills list, keyed by `skill_id`) rather than
 * shown as bare ids.
 */

import { useMemo } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faRotateRight } from "@fortawesome/free-solid-svg-icons";
import { agentToolLabel, type Agent, type Skill } from "../../../lib/claude";

type AgentsPanelProps = {
  agents: Agent[];
  skills: Skill[];
  loading: boolean;
  error: string | null;
  onRefresh: () => void;
};

export function AgentsPanel({ agents, skills, loading, error, onRefresh }: AgentsPanelProps) {
  const skillsById = useMemo(() => new Map(skills.map((skill) => [skill.id, skill])), [skills]);

  return (
    <div className="max-w-3xl space-y-4">
      <div className="flex items-start justify-between gap-2">
        <div>
          <h1 className="text-lg font-semibold text-nest-foreground">Agents</h1>
          <p className="text-xs text-nest-muted">
            Persisted Managed Agents on your Anthropic account, and the skills assigned to each.
          </p>
        </div>
        <button
          type="button"
          onClick={onRefresh}
          disabled={loading}
          title="Refresh from the Claude API"
          className="inline-flex h-7 shrink-0 items-center gap-1 rounded-nest-sm border border-nest-border px-2 text-xs text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground disabled:opacity-50"
        >
          <FontAwesomeIcon icon={faRotateRight} className={["size-3", loading ? "animate-spin" : ""].join(" ")} />
          Refresh
        </button>
      </div>

      {error ? <p className="rounded-nest-sm border border-nest-error/30 bg-nest-error/10 px-3 py-2 text-xs text-nest-error">{error}</p> : null}

      {loading && agents.length === 0 ? (
        <p className="text-xs text-nest-muted">Loading…</p>
      ) : agents.length === 0 && !error ? (
        <p className="text-xs text-nest-muted">No agents yet.</p>
      ) : (
        <div className="space-y-3">
          {agents.map((agent) => (
            <AgentCard key={agent.id} agent={agent} skillsById={skillsById} />
          ))}
        </div>
      )}
    </div>
  );
}

function AgentCard({ agent, skillsById }: { agent: Agent; skillsById: Map<string, Skill> }) {
  return (
    <div className="rounded-nest-md border border-nest-border p-3">
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <div>
          <h2 className="text-sm font-semibold text-nest-foreground">{agent.name}</h2>
          <p className="font-mono text-[11px] text-nest-muted" title={agent.id}>
            {agent.id} · v{agent.version} · {agent.model.id}
          </p>
        </div>
      </div>

      {agent.description ? <p className="mt-2 text-xs text-nest-muted">{agent.description}</p> : null}

      <div className="mt-3">
        <h3 className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-nest-muted">Skills</h3>
        {agent.skills.length === 0 ? (
          <p className="text-xs text-nest-muted">No skills assigned.</p>
        ) : (
          <div className="flex flex-wrap gap-1.5">
            {agent.skills.map((ref) => {
              const skill = skillsById.get(ref.skill_id);
              return (
                <span
                  key={ref.skill_id}
                  title={ref.skill_id}
                  className="inline-flex items-center gap-1 rounded-full border border-nest-border bg-nest-surface px-2 py-0.5 text-[11px] text-nest-foreground"
                >
                  {skill?.display_title ?? ref.skill_id}
                  {ref.version ? <span className="text-nest-muted">v{ref.version}</span> : null}
                </span>
              );
            })}
          </div>
        )}
      </div>

      <div className="mt-3">
        <h3 className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-nest-muted">Tools</h3>
        {agent.tools.length === 0 ? (
          <p className="text-xs text-nest-muted">No tools configured.</p>
        ) : (
          <ul className="space-y-0.5 text-xs text-nest-muted">
            {agent.tools.map((tool, index) => (
              // eslint-disable-next-line react/no-array-index-key -- tools have no stable id
              <li key={index}>{agentToolLabel(tool)}</li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
