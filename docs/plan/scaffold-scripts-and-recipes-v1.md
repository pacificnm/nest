# Scaffold scripts, per-type templates, and recipes v1

## Status: Planned (review only — no implementation yet)

## Context

Kiwi's **File → New Application** wizard (`apps/kiwi/src-tauri/src/new_app.rs`, ~1561
lines) scaffolds all six app types (`gui`, `tui`, `cli`, `system`, `api-server`,
`api-server-web`) with its own from-scratch Rust code generator. It does **not** call
[`scripts/scaffold-desktop-app.sh`](../../scripts/scaffold-desktop-app.sh) or
[`scripts/scaffold-cli-app.sh`](../../scripts/scaffold-cli-app.sh) — those shell scripts
and `new_app.rs` are two independently-maintained scaffolders that happen to overlap on
GUI and CLI.

This has already caused a real bug: this session's work on `templates/desktop/` (fixing
the `run_cli` Tauri ACL "Plugin not found" error, adding the frameless title bar, adding
window-control capabilities) is only reachable through `scaffold-desktop-app.sh`.
`new_app.rs`'s `scaffold_gui_app` copies the same template directory but does its own
separate string substitution, and never picked up the `build.rs` / `capabilities/default.json`
/ `ui/src/lib/nest.ts` fixes. **Any app scaffolded through Kiwi's wizard today reproduces
the exact "Plugin not found" bug this session just fixed.**

Beyond GUI, `new_app.rs` has no template directory at all for `tui`, `system`,
`api-server`, or `api-server-web` — those four are generated from Rust string constants
(`TUI_MAIN_TEMPLATE`, `CLI_MAIN_TEMPLATE`, `SERVICE_MAIN_TEMPLATE`, `API_MAIN_TEMPLATE`,
`API_WEB_MAIN_TEMPLATE`) baked into `new_app.rs` itself, so there is no way to build/test
them independently of the wizard the way `templates/desktop/` and `templates/cli/` can be
built and verified directly.

The wizard's "Nest Core Crates" step (`new_app_list_crates` / `new_app_crate_profile` /
`crate_profile_lists`) is a hand-maintained, per-app-type `(required, recommended)` table
that:

- Only scans `core/crates` — `modules/crates` (where the actual optional integrations
  live: `nest-data-postgres`, `nest-data-sqlite`, `nest-media-library`, `nest-mqtt`,
  `nest-claude`, `nest-ai-ollama`, `nest-airtable`, `nest-tmdb`, `nest-transcode`,
  `nest-cache-file`) is completely invisible to the picker today.
- Only ever affects `Cargo.toml` dependency lines — selecting a crate never generates or
  wires any example code, config, or service registration.
- Has a documented divergence between the profile table's "required" list (cosmetic —
  shown locked in the UI) and the actual hardcoded crate list each `scaffold_*` function
  passes to `merge_crates` (the real structural requirement) — the two are kept in sync
  by hand, with only a code comment enforcing it.

## Goals

1. **One scaffolder per app type, expressed as a shell script + template directory.**
   Kiwi's wizard calls these scripts instead of reimplementing generation logic in Rust.
   `scaffold-desktop-app.sh` (already fixed this session) becomes the reference pattern.
2. **A script + template for every type the wizard currently offers**: `gui` (done),
   `cli` (exists, needs rework — see below), `tui`, `system`, `api-server`,
   `api-server-web`.
3. **Replace the "Nest Core Crates" picker with a recipes system** — standalone scripts
   under `scripts/recipes/` that layer an optional integration (database, media, MQTT,
   AI provider, …) onto an already-scaffolded app, pulling from `modules/crates` (and
   `core/crates` where relevant), driven by a single machine-readable registry instead of
   a hardcoded Rust match.
4. Kiwi's wizard becomes a thin orchestrator: collect name + type + recipe selection,
   shell out to the type's scaffold script, then shell out to each selected recipe
   script, streaming their stdout as progress — no app-type-specific Rust codegen left in
   `new_app.rs`.

## Non-goals (this pass)

- Redesigning the wizard's visual layout/UX beyond swapping the crate picker for a
  recipe picker.
- Retrofitting recipes onto already-scaffolded apps (`apps/kiwi`, `apps/swift`,
  `apps/sparrow`) — this is a "for all future scaffolds" change.
- Changing `new_app_build`'s dormant post-scaffold `cargo check` behavior beyond keeping
  it working (see open question below).
- A general-purpose plugin system for third-party recipes — the initial registry is a
  fixed, repo-maintained list.

## Design

### A. Script + template inventory (target end state)

| App type | Script | Template dir | State today |
|---|---|---|---|
| GUI (Tauri + React) | `scripts/scaffold-desktop-app.sh` | `templates/desktop/` | **Done** — fixed this session (ACL plugin permissions, frameless title bar, `nest.ts` rename gap) |
| CLI | `scripts/scaffold-cli-app.sh` | `templates/cli/` | Exists, but the template is a **flat single-crate app** — doesn't match `app-standard.md`'s documented `crates/core` + `crates/cli` workspace layout, and doesn't match the shape `new_app.rs` currently generates for CLI. Needs rework (below). |
| TUI (Ratatui) | `scripts/scaffold-tui-app.sh` *(new)* | `templates/tui/` *(new)* | Only exists as a Rust string constant (`TUI_MAIN_TEMPLATE`) inside `new_app.rs` |
| System / daemon | `scripts/scaffold-system-app.sh` *(new)* | `templates/system/` *(new)* | Only exists as `SERVICE_MAIN_TEMPLATE` inside `new_app.rs` |
| API server | `scripts/scaffold-api-app.sh` *(new)* | `templates/api-server/` *(new)* | Only exists as `API_MAIN_TEMPLATE` inside `new_app.rs` |
| API + Web | same script, `--web` flag *(new)* | same template + `web/` subtree *(new)* | Only exists as `API_WEB_MAIN_TEMPLATE` + `write_api_web_frontend` inside `new_app.rs` |

**API server vs. API + Web** — one script (`scaffold-api-app.sh <target> [title] [--web]`)
rather than two, since the difference is additive (a `web/` Vite+React subtree plus a
different `main.rs` static-file-serving branch), not a structurally different app. Open
to reconsidering if the two diverge more later.

**Shared scaffold helpers** — `scaffold-desktop-app.sh` and `scaffold-cli-app.sh` already
duplicate ~15 lines of kebab-case-id derivation, title-casing, and placeholder
substitution. Extract this into `scripts/nest-scaffold/lib.sh` (mirroring the existing
`scripts/nest-build/lib.sh` pattern) once a third script needs the same logic, so we're
not maintaining six copies of the same sed pipeline.

**Placeholder convention** — the two existing templates disagree: `templates/desktop/`
replaces the literal string `nest-desktop-template` (i.e., "replace the template's own
name"), while `templates/cli/` uses explicit `{{app_id}}` / mustache-style markers.
Recommend standardizing all *new* templates (`tui`, `system`, `api-server`) on the
explicit `{{app_id}}` / `{{display_title}}` convention — it's greppable and doesn't rely
on the template's placeholder name never colliding with real content. Leave
`templates/desktop/`'s existing convention alone (it was just fixed and re-verified this
session; churning its substitution scheme is a separate, lower-value change). Decide
whether `templates/cli/` gets migrated to the mustache convention as part of its rework
or left as-is.

### B. CLI template rework

`app-standard.md`'s documented CLI layout is:

```text
my-app/
├── crates/
│   ├── core/        # domain logic + services
│   └── cli/
│       └── src/main.rs
└── Cargo.toml        # workspace
```

`templates/cli/` today is a flat single crate (`Cargo.toml` + `src/main.rs` +
`src/cli_command.rs`, no workspace, no `crates/core` split) — it doesn't match the doc,
and it doesn't match the `crates/core` + `crates/cli` shape `new_app.rs`'s
`scaffold_cli_app` currently generates. Recommend reshaping `templates/cli/` to the
documented workspace split so:

- It matches `app-standard.md` (or `app-standard.md` gets corrected instead — pick one;
  recommend fixing the template since the workspace split is genuinely useful once a
  product adds a second surface sharing the same core).
- It's structurally consistent with the new `tui`/`system`/`api-server` templates, which
  all need the same `crates/core` + `crates/<surface>` shape.
- Recipes (below) have one consistent place (`crates/core/Cargo.toml`) to add a
  dependency to, regardless of app type.

### C. Recipes system

**Location:** `scripts/recipes/<recipe-id>.sh`, each taking `<app-dir>` (absolute or
relative to cwd, matching the existing scaffold scripts' argument convention) and
layering a change onto an already-scaffolded app: adding a `modules/crates` (or
`core/crates`) path dependency to `crates/core/Cargo.toml`, and — going further than
today's picker — writing a small example wiring snippet (e.g. a `DataService`
registration call, a commented-out connection string in `config.example.toml`) rather
than only touching `Cargo.toml`. This directly addresses the limitation flagged in the
current picker (crate selection never touches generated code).

**Registry:** `scripts/recipes/registry.json` (or `.toml`) — one machine-readable source
of truth listing recipe id, script path, short label/description, and which app types
it's applicable to (e.g. a database recipe doesn't make sense on a `gui`-only app with no
`crates/core`, unless the GUI app also has a core crate). Kiwi's wizard reads this instead
of the current hardcoded `crate_profile_lists` Rust match. This also gives us a natural
place to note prerequisites (e.g. `database-postgres` assumes a local Postgres instance —
see `scripts/setup-database-postgres.sh` for the closest existing precedent of a
setup-style script in this repo).

**Initial recipe set** (start with 2–3 as a proof of concept before building out the
rest — see Rollout phasing):

| Recipe id | Wraps | Notes |
|---|---|---|
| `database-postgres` | `nest-data-postgres` | Pairs with `scripts/setup-database-postgres.sh` for the actual DB provisioning |
| `database-sqlite` | `nest-data-sqlite` | No external service needed |
| `cache-file` | `nest-cache-file` | Already used by `templates/desktop/` directly — good first recipe to validate the pattern against a known-working integration |
| `media-library` | `nest-media-library` | |
| `mqtt` | `nest-mqtt` | Recently landed (`Issue 12.1` TLS work on `feat/nest-mqtt-tls-config`) |
| `ai-ollama` | `nest-ai-ollama` | |
| `ai-claude` | `nest-claude` / `nest-ai-claude` | |
| `airtable` | `nest-airtable` | |
| `transcode` | `nest-transcode` | |
| `tmdb` | `nest-tmdb` | |

**Idempotency / tracking:** each recipe script should refuse (with a clear message) to
re-apply itself if already applied, and should record what's been applied — recommend a
lightweight `.nest-recipes` manifest file (one recipe id per line) written into the app
root, both so re-runs are safe and so a future "add recipe to an existing app" wizard flow
(out of scope here, but worth not blocking) has something to read.

### D. Kiwi wizard changes

- `new_app_scaffold` (`apps/kiwi/src-tauri/src/new_app.rs`) becomes a thin orchestrator:
  `Command::new(scaffold_script_for(app_type)).arg(&target).arg(&title).status()`, then
  for each selected recipe, `Command::new("scripts/recipes/<id>.sh").arg(&target).status()`
  — reusing the existing `progress: &dyn Fn(&str)` / `emit_progress` /
  `new-app://progress` event plumbing, fed from subprocess stdout lines instead of
  internally-formatted strings.
- Delete the six `scaffold_*` codegen functions and their string-constant templates
  (`TUI_MAIN_TEMPLATE`, `CLI_MAIN_TEMPLATE`, `SERVICE_MAIN_TEMPLATE`,
  `API_MAIN_TEMPLATE`, `API_WEB_MAIN_TEMPLATE`, `WEB_PACKAGE_JSON`, etc.) once the
  corresponding script + template exists and is verified — this is the bulk of
  `new_app.rs`'s ~1561 lines.
- `new_app_list_crates` / `new_app_crate_profile` / `crate_profile_lists` are replaced by
  a `new_app_list_recipes(app_type)` command reading `scripts/recipes/registry.json`.
- **Path-patching moves into the scripts.** Today `new_app.rs` writes the GUI app's
  `.cargo/config.toml` `[patch.crates-io]` block and the other types'
  `[workspace.dependencies]` tables in Rust. This logic has to live in the scaffold
  scripts (writing the template's own baseline deps) and the recipe scripts (appending
  their crate's path dependency on top) — flagging explicitly since this was the fiddliest
  part of the current Rust implementation and is the easiest thing to lose in translation.
- `new_app_build`'s post-scaffold `cargo check` (`run_cargo_check`) is orthogonal to the
  script-vs-Rust-codegen question and can be kept as-is, calling it after the new
  orchestrated `new_app_scaffold` — see open question below on whether to keep it at all.
- Frontend `NewAppWizard.tsx` / `lib/newApp.ts`: swap the three-tier "Nest Core Crates"
  checkbox UI for a flat "Recipes" checkbox list sourced from `new_app_list_recipes`;
  rename `selectedCrates` → `selectedRecipes` end-to-end (Rust request struct, IPC
  payload, frontend state); update the `CrateProfile` type to whatever shape the registry
  read returns.

### E. Verification approach

Mirror how `templates/desktop/`'s fixes were verified this session, for each new
script/template:

1. Run the script standalone against a scratch target dir.
2. `./build build` (or `cargo check` for non-Tauri types) on the result.
3. For GUI specifically: confirm the ACL manifest (`gen/schemas/acl-manifests.json`) has
   a real entry for the app's plugin id, and the compiled JS bundle's `plugin:<id>|...`
   invoke strings match — this is exactly the class of bug this plan exists to prevent
   recurring.
4. Only after a script/template is independently verified does `new_app.rs` get switched
   to call it (and its Rust codegen deleted) — avoids a window where the wizard is broken
   for a type mid-migration.

## Rollout phasing

1. **Phase 1 — shared scaffold lib + CLI template rework.** Extract
   `scripts/nest-scaffold/lib.sh`; reshape `templates/cli/` to the `crates/core` +
   `crates/cli` workspace split; align or correct `app-standard.md`. Small, self-contained,
   de-risks the pattern before the bigger phases.
2. **Phase 2 — new templates + scripts.** One sub-phase per type: `tui`, `system`,
   `api-server`(+`--web`). Each independently buildable/testable via `./build` without
   touching Kiwi.
3. **Phase 3 — recipes system.** Registry format, recipe script conventions, and the
   first 2–3 recipes (`cache-file`, `database-sqlite`, `database-postgres`) as a proof of
   concept before building out the remaining ~7.
4. **Phase 4 — Kiwi wizard rewrite.** `new_app.rs` becomes a subprocess orchestrator;
   frontend recipe picker; delete dead per-type codegen once each script is verified
   (Phase 2/3 must land first per type).
5. **Phase 5 — cleanup.** Remove now-dead code/types, fix the stale "ribbon" mention in
   `templates/desktop/README.md` (predates this session's Ribbon removal), sweep
   `app-standard.md` / `apps/README.md` for any other drift.

## Open questions (need a decision before implementation starts)

1. **Recipe depth:** Cargo.toml-only dependency addition (matches today's picker
   behavior, less work per recipe) vs. real example wiring code (addresses the picker's
   documented limitation, more work per recipe, more valuable). Leaning toward "real
   wiring" for the first 2–3 recipes as a proof of concept, revisit for the rest.
2. **`templates/cli/` migration:** reshape to workspace+core split (recommended, matches
   docs and the other new templates) vs. leave flat and correct the docs instead.
3. **`api-server` vs `api-server-web`:** one script with a `--web` flag (recommended) vs.
   two separate scripts/templates.
4. **`new_app_build` / cargo-check-after-scaffold:** keep (low cost, currently
   unused-by-UI but harmless) vs. remove since nothing calls it today.
5. **Placeholder convention for `templates/cli/`:** migrate to `{{mustache}}` style to
   match the new templates, or leave its existing literal-string-replace convention.

## Related

- [templates/desktop/](../../templates/desktop/) — reference pattern (GUI scaffold, just
  fixed this session)
- [scripts/scaffold-desktop-app.sh](../../scripts/scaffold-desktop-app.sh)
- [scripts/scaffold-cli-app.sh](../../scripts/scaffold-cli-app.sh)
- [scripts/nest-build/lib.sh](../../scripts/nest-build/lib.sh) — precedent for a shared
  shell helper library
- [apps/kiwi/docs/plan/kiwi-tauri-v1.md](../../apps/kiwi/docs/plan/kiwi-tauri-v1.md)
- [app-standard.md](../app-standard.md) — folder layouts referenced throughout
- [build.md](../build.md) — `./build` verb/profile reference
