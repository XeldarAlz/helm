# Rubric: test-writing/write-score-tests

## Criteria (1-5 each)

1. **Coverage** — Are all public methods tested? AddScore, GetTopScores, GetPlayerScore, ResetScores, TotalEntries?
2. **Edge Cases** — Boundary values (0, negative, max), empty state, full capacity?
3. **Error Paths** — Negative score throws? Adding beyond MaxEntries handled?
4. **Test Structure** — AAA pattern, one assertion per test, descriptive Method_Scenario_Expected names?
5. **Fakes Quality** — Hand-rolled FakeScoreConfig/FakeScoreEvents? Clean, minimal, reusable?
6. **Independence** — Tests don't depend on each other? Fresh state per test via [SetUp]?
