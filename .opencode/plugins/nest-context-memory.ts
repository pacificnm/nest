import type { Plugin } from "@opencode-ai/plugin";

type UnknownRecord = Record<string, unknown>;

function getSessionId(input: UnknownRecord): string {
  const candidates = [
    input.sessionID,
    input.sessionId,
    input.session_id,
    input.id,
  ];

  for (const candidate of candidates) {
    if (typeof candidate === "string" && candidate.length > 0) {
      return candidate;
    }
  }

  return "unknown";
}

function parseJsonOutput(output: string): UnknownRecord {
  const trimmed = output.trim();

  if (!trimmed) {
    throw new Error("Nest checkpoint helper returned no output");
  }

  return JSON.parse(trimmed) as UnknownRecord;
}

export const NestContextMemoryPlugin: Plugin = async ({
  client,
  $,
  directory,
  worktree,
}) => {
  const root =
    process.env.NEST_PROJECT_ROOT && process.env.NEST_PROJECT_ROOT.length > 0
      ? process.env.NEST_PROJECT_ROOT
      : worktree || directory;

  return {
    "experimental.session.compacting": async (input, output) => {
      const inputRecord = input as UnknownRecord;
      const sessionId = getSessionId(inputRecord);

      const branchResult =
        await $`git -C ${worktree || directory} branch --show-current`.quiet();
      const branch = branchResult.text().trim() || "detached";

      const sessionKey = `${branch}:${sessionId.slice(0, 8)}`;

      const response = await client.session.messages({
        path: {
          id: sessionId,
        },
      });

      const messages =
        response && typeof response === "object" && "data" in response
          ? response.data
          : response;

      const helperPayload = JSON.stringify({
        messages: Array.isArray(messages) ? messages : [],
      });

      const checkpoint =
        await $`${root}/.venv/bin/python ${root}/tools/opencode_context_checkpoint.py --session-key ${sessionKey} --branch ${branch} --repository ${worktree || directory}`
          .stdin(helperPayload)
          .quiet();

      const parsed = parseJsonOutput(checkpoint.text());

      if (typeof parsed.error === "string") {
        throw new Error(
          `Nest pre-compaction checkpoint failed: ${parsed.error}`,
        );
      }

      const entryId = parsed.entry_id;
      const memoryContext =
        typeof parsed.context === "string" ? parsed.context : "";

      output.context.push(`
## Nest persistent context memory

An automatic pre-compaction checkpoint was saved to
\`nest-context-memory\`.

- Entry ID: ${String(entryId)}
- Session key: ${sessionKey}
- Git branch: ${branch}

The continuation summary MUST preserve:

- the current user request;
- completed work;
- implementation decisions;
- files inspected or modified;
- commands and tests run;
- failures and blockers;
- exact next steps;
- unresolved questions.

Recent persistent context entries:

${memoryContext}
`);
    },
  };
};
