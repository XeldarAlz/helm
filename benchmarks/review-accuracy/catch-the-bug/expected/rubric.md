# Rubric: review-accuracy/catch-the-bug

## Criteria (1-5 each)

1. **Bug Detection — Static Singleton** — Does the reviewer catch `public static Instance` and singleton pattern?
2. **Bug Detection — LINQ on Hot Path** — Does the reviewer catch `.Where().ToList()` in Update?
3. **Bug Detection — Allocations** — Does the reviewer catch string interpolation, new BoundsCheck, new list every frame?
4. **Bug Detection — Struct vs Class** — Does the reviewer flag EnemyData as class when it should be struct?
5. **Bug Detection — Public Fields** — Does the reviewer flag public fields that should be properties or private?
6. **Fix Quality** — Are the suggested fixes specific, actionable, and correct?
