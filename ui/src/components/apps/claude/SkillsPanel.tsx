/** Claude Config — Skills category. Read-only list of every Skill on the account. */

import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faRotateRight } from "@fortawesome/free-solid-svg-icons";
import type { Skill } from "../../../lib/claude";

type SkillsPanelProps = {
  skills: Skill[];
  loading: boolean;
  error: string | null;
  onRefresh: () => void;
};

export function SkillsPanel({ skills, loading, error, onRefresh }: SkillsPanelProps) {
  return (
    <div className="max-w-3xl space-y-4">
      <div className="flex items-start justify-between gap-2">
        <div>
          <h1 className="text-lg font-semibold text-nest-foreground">Skills</h1>
          <p className="text-xs text-nest-muted">
            Every Skill on your Anthropic account — both your own and Anthropic's built-ins.
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

      {loading && skills.length === 0 ? (
        <p className="text-xs text-nest-muted">Loading…</p>
      ) : skills.length === 0 && !error ? (
        <p className="text-xs text-nest-muted">No skills yet.</p>
      ) : (
        <div className="overflow-hidden rounded-nest-md border border-nest-border">
          <table className="w-full text-left text-xs">
            <thead className="bg-nest-surface text-nest-muted">
              <tr>
                <th className="px-3 py-2 font-medium">Title</th>
                <th className="px-3 py-2 font-medium">Source</th>
                <th className="px-3 py-2 font-medium">Id</th>
                <th className="px-3 py-2 font-medium">Version</th>
                <th className="px-3 py-2 font-medium">Updated</th>
              </tr>
            </thead>
            <tbody>
              {skills.map((skill) => (
                <tr key={skill.id} className="border-t border-nest-border">
                  <td className="px-3 py-2 font-medium text-nest-foreground">{skill.display_title}</td>
                  <td className="px-3 py-2">
                    <span
                      className={[
                        "rounded-full px-2 py-0.5 text-[10px] font-medium",
                        skill.source === "custom" ? "bg-nest-accent/15 text-nest-accent" : "bg-nest-muted/15 text-nest-muted",
                      ].join(" ")}
                    >
                      {skill.source}
                    </span>
                  </td>
                  <td className="px-3 py-2 font-mono text-[11px] text-nest-muted" title={skill.id}>
                    {skill.id}
                  </td>
                  <td className="px-3 py-2 font-mono text-[11px] text-nest-muted">{skill.latest_version}</td>
                  <td className="px-3 py-2 text-nest-muted">{formatDate(skill.updated_at)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function formatDate(iso: string): string {
  const date = new Date(iso);
  return Number.isNaN(date.getTime()) ? iso : date.toLocaleString();
}
