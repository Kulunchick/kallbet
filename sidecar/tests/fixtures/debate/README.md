# Debate fixtures (ticket #29)

Real output of the installed, signed-in CLIs for one run of the debate lifecycle, captured on 2026-11-25 on Windows 11 with `claude` 2.1.332, `codex-cli` 0.172.0 and `agy` 1.5.17. Nothing was edited. No file holds a credential.

## How they were produced

```
DEBATE_CAPTURE_DIR=<dir> cargo test -p kallbet-ai --test debate_live -- --ignored --nocapture
```

`tests/debate_live.rs` runs the scenario of `tests/debate_support/mod.rs` through the real CLIs with the flags the adapters of ticket #27 build (no tools, the seat's JSON Schema), and writes for every CLI run its prompt (`*.prompt.txt`: stdin for `claude` and `codex`, the `--print=` argument for `agy`), its stdout (`*.json`, `*.jsonl`) and, when not empty, its stderr (`*.stderr.txt`).

The scenario: Vitality meet MOUZ in a Bo3; the market makes Vitality a 52/48 favourite and the scripted rating estimate gives them 45%, so the anchor is 50.6% for Vitality. The Draft is made four hours ahead with no context (Liquipedia unreachable). At T-22m the context holds the roster facts of the saved revision `fixtures/context/liquipedia/pgl-singapore-2026-rev-before-start.json` (Vitality field apEX for the benched jL) and the roster diff against `fixtures/vrs/standings_global_2026_11_02.md`. The bot's `provider_chain` is `claude`, `codex`, `antigravity`, which seats Claude as Market Advocate, Codex as Devil's Advocate and Antigravity as Chief Arbiter.

## The runs

| File | Seat and CLI | Exit | What it answered |
|---|---|---|---|
| `predict-claude.json` | the Draft's single agent, `claude` | 0 | `delta` 0, no factors |
| `advocate-claude.json` | Market Advocate, `claude` | 0 | brief, `delta` -0.5 |
| `devil-codex.jsonl` | Devil's Advocate, `codex exec` | 0 | brief, `delta` -2 |
| `arbiter-agy.json`, `arbiter-agy.stderr.txt` | Chief Arbiter, `agy` | 3 | nothing: the Antigravity subscription was out of quota ("Individual quota reached", `RESOURCE_EXHAUSTED`, code 429). This was not provoked; it is what the account answered at capture time. |
| `arbiter-claude.json` | Chief Arbiter, `claude`, the next provider of that seat's chain | 0 | `delta` -1.5 with one `stand_in` factor citing the Liquipedia revision |

So the capture is itself a debate in which one seat's subscription ran out and the seat failed over. `agy` therefore never answered a debate prompt here; its acceptance of the seats' schemas is covered only by the general smoke test of ticket #27 (`tests/llm_live.rs`).

`arbiter-agy.json` with its stderr is also the first real quota failure of `agy` in the repository: ticket #27 derived that case from the binary's strings (see `fixtures/llm/README.md`).

## Use

`tests/debate_sidecar.rs` replays these files by seat and CLI through a scripted `ProcessRunner`. The replay does not compare prompts, so the `*.prompt.txt` files only document what each answer was given; after a change to the seats' instructions or to the dossier, capture again to keep them true.
