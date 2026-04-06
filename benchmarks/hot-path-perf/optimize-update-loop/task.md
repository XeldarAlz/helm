## Task Assignment

**Task ID:** BENCH-HP-1
**Task Title:** Implement ScoreProcessor with Zero-Allocation Update Loop
**Description:** Implement the `ScoreProcessor` class that processes up to 1000 score entries per frame. This runs every frame in FixedUpdate, so it MUST have zero heap allocations. Use pre-allocated arrays, Span<T>, or stackalloc for all temporary buffers.

**Output Files:**
- `Assets/Scripts/Logic/Score/ScoreProcessor.cs`

**Acceptance Criteria:**
- Implements `IScoreProcessor` fully
- Process method handles up to 1000 entries per call with zero allocation
- Uses pre-allocated arrays (initialized in constructor)
- No LINQ, no `new` for reference types, no string operations in Process
- Uses struct-based ScoreEntry (not class)
- Batch processing with configurable batch size
- Results written to a pre-allocated output buffer (not returned as new collection)
