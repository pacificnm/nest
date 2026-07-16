# Before ending any response

Call `nest-context-memory_save_context_memory` before the turn ends —
after read-only investigations as well as after implementation turns. There
is no hook that blocks the response if this is skipped, so treat it as
mandatory self-discipline rather than something enforced for you.

Include:

- what you did this turn;
- files changed or read;
- decisions and blockers;
- verification commands and results;
- exact next steps, if the task isn't finished.

Use the current git branch as `session_key` (same key used at the start of
the turn — see [memory-workflow.md](memory-workflow.md)). If the save fails,
say so in your response rather than silently continuing.
