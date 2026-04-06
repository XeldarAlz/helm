## Task Assignment

**Task ID:** BENCH-TW-1
**Task Title:** Write Unit Tests for ScoreSystem
**Description:** Write comprehensive NUnit unit tests for the `ScoreSystem` class. Cover happy paths, edge cases, and error paths. Use hand-rolled fakes for dependencies.

**Output Files:**
- `Assets/Tests/Unit/ScoreSystemTests.cs`
- `Assets/Tests/Fakes/FakeScoreConfig.cs`

**Acceptance Criteria:**
- Tests for AddScore: valid score, zero score, negative score, max int
- Tests for GetTopScores: empty, single entry, full list, more than requested
- Tests for ResetScores: clears all, can add after reset
- Tests for event firing: OnScoreAdded fires with correct data, OnScoresReset fires
- Edge cases: concurrent-style access patterns, boundary values
- All tests use Arrange-Act-Assert
- One assertion per test method
- Descriptive names: Method_Scenario_Expected
- Hand-rolled FakeScoreConfig (no Moq/NSubstitute)
