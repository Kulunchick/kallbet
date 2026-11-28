# LLM track of the Signal Lab

Not run: no `--llm` file and no `--answers`. To run it (`<RUNS>` is the budget of CLI runs you choose for each subscription):

```
cargo run -p kallbet-ai -- lab --archive <ARCHIVE> --llm <LLM_FILE> --answers <ANSWERS_FILE> --budget claude=<RUNS> --budget codex=<RUNS> --budget antigravity=<RUNS>
```
