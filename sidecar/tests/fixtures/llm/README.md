# LLM runtime fixtures (ticket #27)

Real output of the installed CLIs and real HTTP answers, captured on 2026-11-25 on Windows 11 with `claude` 2.1.332, `codex-cli` 0.172.0 and `agy` (build of 2026-11-25). Files from the spec #21 research archive (`.spec21-notes/fixtures/cli/`) were converted from UTF-16/UTF-8-BOM to UTF-8; nothing else was edited. No file holds a credential.

`tiny-schema.json` is the schema every CLI capture was asked to fill: `{"sum": integer}`, prompt "What is 2+3? Return JSON matching the schema."

## Successful runs (research archive)

| File | Command (stdin closed) | Exit |
|---|---|---|
| `claude-success.json` | `claude -p "<prompt>" --output-format json --json-schema <tiny-schema> --tools "" --strict-mcp-config --disable-slash-commands --no-session-persistence --permission-prompts none` | 0 |
| `codex-success.jsonl` | `codex exec "<prompt>" --ephemeral --skip-git-repo-check -s read-only -c 'web_search="disabled"' --json --output-schema tiny-schema.json` | 0 |
| `agy-success.json` | `agy -p "<prompt>" --output-format json --json-schema tiny-schema.json --disable-slash-commands --print-timeout 60s` | 0 |

## Failures captured for this ticket

| File | How it was produced | Exit |
|---|---|---|
| `claude-badschema.stderr.txt` | stderr of `claude -p hi ... --json-schema '{not json'` (same flags as above) | 1 |
| `claude-signedout.json` | stdout of the success command with `CLAUDE_CONFIG_DIR` set to an empty directory (no login) | 1 |
| `claude-quota.json` | stdout of the success command with `CLAUDE_CODE_OAUTH_TOKEN=sk-ant-oat01-dummy`, an empty `CLAUDE_CONFIG_DIR` and `ANTHROPIC_BASE_URL` pointing at a local server that answers every `/v1/messages` with the HTTP 429 (headers and body) of a real weekly-limit run, as published in anthropics/claude-code#103820 (mock `experiments/issue-3570/mock-weekly-limit-server.mjs` of link-assistant/hive-mind, branch `issue-3570-e9ebcaf8355f`). The envelope is what the real CLI printed for it. | 1 |
| `codex-signedout.jsonl` | stdout of the success command with `CODEX_HOME` set to an empty directory | 1 |
| `codex-quota.jsonl` | stdout of the success command with an empty `CODEX_HOME` and a provider override `-c model_provider="mock" -c 'model_providers.mock={name="mock",base_url="http://127.0.0.1:8794/v1",wire_api="responses",env_key="MOCK_KEY",request_max_retries=0,stream_max_retries=0}'` pointing at a local server answering `POST /v1/responses` with 429 `{"error":{"type":"usage_limit_reached","message":"The usage limit has been reached","plan_type":"plus","resets_at":1792494000}}` (the ChatGPT backend's usage-limit shape that `codex.exe` parses). The JSONL and the "You’ve hit your usage limit" text are the real CLI's. | 1 |
| `codex-badschema.stderr.txt` | stderr of `codex exec hi --ephemeral --skip-git-repo-check -s read-only --json --output-schema does-not-exist.json` | 1 |
| `agy-timeout.json`, `agy-timeout.stderr.txt` | stdout and stderr of the success command with `--print-timeout 1ms` | 3 |
| `agy-badmodel.json` | stdout of `agy -p "What is 2+3?" --output-format json --json-schema tiny-schema.json --disable-slash-commands --model no-such-model` | 1 |
| `anthropic-401.json` | `curl https://api.anthropic.com/v1/messages` without a key (`anthropic-version: 2023-06-01`) | HTTP 401 |
| `openai-401.json` | `curl https://api.openai.com/v1/chat/completions` without a key | HTTP 401 |

## HTTP success bodies (provider API reference samples)

No Anthropic or OpenAI API key is available on this machine, so the 200 bodies are the samples the providers publish, saved verbatim:

| File | Source |
|---|---|
| `anthropic-message.json` | "Response (200)" of `POST /v1/messages`, https://platform.claude.com/docs/en/api/messages/create (fetched 2026-11-25) |
| `openai-chat-completion.json` | Default example response of `POST /v1/chat/completions`, https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create (fetched 2026-11-25) |

Their text answers are plain prose, so as saved they are a non-JSON answer (`InvalidOutput`); the tests derive the structured answer by replacing that text with `{"sum":5}`.

## Derived in tests, not saved

- **agy quota**: no quota run of `agy` was provoked. Tests take `agy-timeout.json` and replace `error` with "You have exhausted your quota on this model.", a message `agy.exe` carries (found with `grep -a` in the binary, next to "model %s is out of quota until %s" and "Usage limit reached"). The `AGY_ERROR` stderr line's `status` field (`DEADLINE_EXCEEDED` in the timeout capture) is read as a gRPC status name, so `RESOURCE_EXHAUSTED` is taken as quota; that mapping is unverified against a real quota run.
- **claude session limit**: `claude-quota.json` with `result` replaced by "You've hit your session limit · resets 7pm (UTC)"; "You've hit your session limit", "weekly limit" and "monthly limit" are strings in the `claude` binary.
- Non-JSON codex answer: `codex-success.jsonl` with the agent message replaced by `five`.
