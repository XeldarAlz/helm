# Benchmark Suite

Self-contained tasks for regression-testing agent templates, skills, and model routing. Each benchmark runs an agent against a standardized task and scores the output.

## Running

```
/benchmark                                    — Run all benchmarks
/benchmark pure-logic                         — Run one category
/benchmark pure-logic/implement-inventory     — Run one specific benchmark
```

## Adding a Benchmark

1. Create a directory under the appropriate category in `benchmarks/`
2. Add `config.md`, `task.md`, `input/`, and `expected/` following existing examples
3. Run `/benchmark <your-benchmark>` to verify it works

## Categories

| Category | Tests | Purpose |
|----------|-------|---------|
| pure-logic | Coder agent implementation quality | Core C# logic without Unity deps |
| hot-path-perf | Performance constraint compliance | Zero-allocation patterns |
| test-writing | Tester agent quality | Test coverage, structure, naming |
| review-accuracy | Reviewer agent detection ability | Catches deliberate bugs/violations |
| interface-design | Architecture quality | Clean interfaces, SOLID compliance |

## Scoring

- **Automated:** File existence, pattern presence (grep), anti-pattern absence → pass/fail
- **Qualitative:** Benchmark runner (opus) evaluates against rubric → 1-5 per criterion
- **Verdict:** PASS requires all automated checks pass + rubric avg >= 3.0

## Results

Accumulated results are appended to `benchmarks/RESULTS.md`. Compare across runs to detect regressions from prompt/template changes.
