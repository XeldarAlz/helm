# Rubric: hot-path-perf/optimize-update-loop

## Criteria (1-5 each)

1. **Zero Allocation** — Is the Process method completely allocation-free? No new, no boxing, no LINQ, no closures?
2. **Pre-allocation** — Are all buffers allocated in the constructor? Is capacity configurable via IScoreConfig?
3. **Span Usage** — Does it correctly use ReadOnlySpan/Span for input/output? No unnecessary copies?
4. **Batch Processing** — Is batch size respected? Does it process in chunks efficiently?
5. **Correctness** — Does it actually compute correct scores (rawScore * multiplier) and rankings?
6. **Code Quality** — Clean structure, no XML docs, proper naming, sealed class?
