# nest-auth / nest-auth-oauth-client — Plan

## Status: Planned (review only — no implementation yet)

## Context

Project Finch (`apps/finch`, a local trading-research assistant — see
[`apps/finch/docs/trading-research-assistant-plan.md`](../../apps/finch/docs/trading-research-assistant-plan.md))
needs to talk to the Charles Schwab Trader API, which is OAuth2-only. There is
currently no auth abstraction anywhere in the framework: `nest-http`'s
`AuthStrategy` trait (`core/crates/nest-http/src/auth.rs`) only covers
*applying* an already-obtained credential to an outgoing request
(`BearerTokenAuth` sets a header) — it has no concept of acquiring a token,
refreshing one, or storing one. Grepped the whole tree for `oauth`/`OAuth`:
the only hits are `apps/kiwi/src-tauri/src/accounts.rs`, which doesn't
implement OAuth itself — `codex_login` just shells out to the external
`codex` CLI's own `codex login` and reads the status file it writes. There is
no in-process OAuth client anywhere to extract from or build on; this is new,
foundational work.

## Goals

1. A **core crate** (`nest-auth`) defining the mechanism-agnostic vocabulary
   every future auth integration shares: a token representation and secure
   storage for it. Not a unified "auth provider" trait — see the explicit
   non-goal below for why.
2. A **module crate** (`nest-auth-oauth-client`) implementing the actual
   OAuth2 authorization-code + PKCE flow, built on `nest-auth`'s `TokenStore`
   and `nest-http`'s existing `AuthStrategy`, so acquired tokens flow
   straight into the HTTP call path every other module already uses.
3. Schwab as the first real, driving consumer — Finch is what proves the
   design out, the same way Ollama was `nest-ai`'s first real provider before
   Claude validated the abstraction was actually reusable.
4. A `scripts/recipes/oauth-client.sh` recipe once the crate is built and
   verified, so pulling it into a product matches the same
   `database-postgres`/`http-client` pattern already established.

## Non-goal: no unified `AuthProvider` trait, no oauth-server, no password auth yet

`nest-ai`'s `AiProvider` trait works as *one* trait because every provider
does the same thing (complete a prompt) — `nest-ai-ollama` and a
hypothetical `nest-ai-openai` are interchangeable behind it. OAuth-client,
OAuth-server, and password auth are not interchangeable that way: acquiring
a token from someone else's authorization server, issuing your own tokens,
and verifying a local password are structurally different operations with
no natural shared method signature. Forcing them under one trait now means
guessing at a shape with only one real consumer to validate against — the
same trap `nest-ai` avoided by not finalizing its trait until a second
provider existed. `nest-auth-oauth-server` and `nest-auth-password` are
explicitly deferred until something real needs them.

## Architecture

Mirrors the existing `nest-ai` / `nest-ai-ollama` and `nest-data` /
`nest-data-postgres` pattern exactly — same `core/crates` vs `modules/crates`
split, same `nest_core::Module` (`id()` / `dependencies()` / `configure()`)
registration shape already used by `OllamaModule`
(`modules/crates/nest-ai-ollama/src/module.rs`). App code depends on the
core crate's service type, never on the concrete module, the same way
`crates/core` in `nest-ai`-based apps depends on `AiService`, never on
`OllamaProvider` directly.

### `nest-auth` (`core/crates/nest-auth`)

Deliberately minimal — just the pieces that are genuinely mechanism-agnostic:

- **`Token`** — access token, optional refresh token, expiry, scope. Plain
  data, no behavior.
- **`TokenStore`** trait — `get`/`put`/`delete` a `Token` by a string key
  (e.g. `"schwab"`), `async_trait` to match every other service trait in the
  framework (`AiProvider`, etc.). One reference implementation ships here: a
  file-backed store (gitignored path under the app's own data directory) —
  good enough for local dev, explicitly *not* the recommended production
  answer (see Open Questions).
- Error type (`AuthError`) following the same `thiserror`-based shape as
  `AiError`/`NestError`.

No `Module` in this crate — same reasoning as `nest-ai` not registering a
default provider: there's nothing sensible to register without a concrete
implementation choosing *how* tokens are acquired.

### `nest-auth-oauth-client` (`modules/crates/nest-auth-oauth-client`)

- Built on the [`oauth2`](https://docs.rs/oauth2) crate for the actual RFC
  6749 + PKCE mechanics (authorization URL construction, code exchange,
  refresh) rather than hand-rolling the wire protocol.
- Once a token is acquired, wraps it in something implementing `nest-http`'s
  existing `AuthStrategy` (`core/crates/nest-http/src/auth.rs`) so
  authenticated calls flow through the same `nest-http-client` pipeline
  every other module already uses — no parallel HTTP path.
- `OAuthClientModule` (mirrors `OllamaModule` exactly): config loaded from a
  TOML section via `ConfigService` (client id, authorization/token
  endpoints, redirect URI, scopes) with an explicit-config constructor for
  tests; `dependencies()` declares `HTTP_CLIENT_MODULE_ID`; `configure()`
  builds the client and calls `app.register_service(...)`.
- **Redirect handling** — this is a desktop app, not a web server, so the
  provider's OAuth redirect has nowhere to land by default. Plan: a
  short-lived local loopback HTTP listener
  (`http://127.0.0.1:<port>/callback`), opened only for the duration of the
  login flow, same approach most desktop OAuth clients use (e.g. `gh auth
  login`). A custom URI scheme deep-link is the alternative; loopback is
  simpler and doesn't need OS-level URI-scheme registration, so it's the
  default plan unless something concrete rules it out.
- **Schwab-specific behavior**, kept inside this crate (not a further
  `nest-auth-oauth-schwab` split) until a second OAuth provider is a real
  requirement:
  - Schwab's refresh tokens expire in 7 days — `TokenStore`/the module
    should surface "needs re-auth" as a distinct, checkable state rather
    than folding it into a generic auth error, so Finch's UI can prompt for
    re-login proactively instead of failing an API call and guessing why.
  - Schwab's exact endpoints/scopes live in this crate's config defaults,
    not hardcoded into `nest-auth` itself.

## Rollout phasing

1. **`nest-auth`** — `Token`, `TokenStore` trait + file-backed reference
   impl, error type. No external API calls, straightforward to land and
   test in isolation.
2. **`nest-auth-oauth-client`**, generic RFC 6749 + PKCE flow and the
   loopback redirect listener, tested against a throwaway/mock OAuth
   provider (not Schwab directly) so the flow itself is verified before
   spending a real Schwab developer app registration on it.
3. **Schwab wiring** — real endpoints/scopes, `database-postgres`-style
   `apply_finch/http-client` recipe, first true end-to-end login against
   Schwab's actual authorization server.
4. **`scripts/recipes/oauth-client.sh`** once 1–3 are verified, registered
   in `scripts/recipes/registry.json` alongside `database-postgres` /
   `database-sqlite` / `http-client` / `ai`.
5. **Revisit `nest-auth-oauth-server` / `nest-auth-password`** only when a
   real product needs one — not scheduled here.

## Open questions (need a decision before implementation starts)

1. **Production token storage.** File-backed `TokenStore` is fine for local
   dev, but these are real brokerage credentials — the real answer is
   almost certainly an OS-keyring-backed implementation (the `keyring`
   crate covers Linux/macOS/Windows). Decide whether that ships as part of
   `nest-auth-oauth-client` v1 or is a fast-follow once the file-backed path
   is proven.
2. **Loopback listener port.** Fixed port (simpler, but can collide with
   something else already listening) vs. OS-assigned ephemeral port
   (registered with Schwab's app config as a range, if their dashboard
   supports that — needs checking against Schwab's actual app registration
   UI).
3. **Multi-account.** Does Finch ever need more than one Schwab account
   authenticated at once? Affects whether `TokenStore`'s key is just
   `"schwab"` or needs to be provider+account-scoped from the start.
4. **Token refresh trigger.** Proactive background refresh before expiry vs.
   lazy refresh-on-401. Lazy is simpler; proactive avoids a failed API call
   being the first sign a refresh was needed.

## Related

- [`apps/finch/docs/trading-research-assistant-plan.md`](../../apps/finch/docs/trading-research-assistant-plan.md) — Finch's own plan, the driving use case
- [`core/crates/nest-ai`](../../core/crates/nest-ai) / [`modules/crates/nest-ai-ollama`](../../modules/crates/nest-ai-ollama) — the core-crate + module-crate pattern this mirrors
- [`core/crates/nest-data`](../../core/crates/nest-data) / [`modules/crates/nest-data-postgres`](../../modules/crates/nest-data-postgres) — second precedent for the same split
- [`core/crates/nest-http/src/auth.rs`](../../core/crates/nest-http/src/auth.rs) — existing `AuthStrategy` trait this integrates with, not replaces
- [`docs/plan/scaffold-scripts-and-recipes-v1.md`](../plan/scaffold-scripts-and-recipes-v1.md) — the recipes system `oauth-client.sh` will plug into
- [`apps/kiwi/src-tauri/src/accounts.rs`](../../apps/kiwi/src-tauri/src/accounts.rs) — checked for existing OAuth precedent; confirmed there isn't any (delegates to external CLIs instead)
