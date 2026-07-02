# nest-ai-ollama

Ollama adapter for [nest-ai](../nest-ai/README.md).

**Crate path:** [`modules/crates/nest-ai-ollama`](../../modules/crates/nest-ai-ollama)

## Usage

```toml
[ai]
enabled = true
provider = "ollama"
base_url = "http://127.0.0.1:11434"
model = "smollm2:360m"
```

Ollama must be running (`ollama serve`) and reachable from the app host. Default bind is `http://127.0.0.1:11434`; remote hosts work when configured and firewalled appropriately.

## API

Uses Ollama chat endpoint (`POST /api/chat`) with optional JSON response format.

## Related

- [nest-ai](../nest-ai/README.md)
