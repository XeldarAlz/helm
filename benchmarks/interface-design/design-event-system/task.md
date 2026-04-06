## Task Assignment

**Task ID:** BENCH-ID-1
**Task Title:** Implement Type-Safe Event Bus
**Description:** Implement the `EventBus` class that fulfills the `IEventBus` interface. The event bus must be type-safe, support subscribe/unsubscribe/publish, and have zero allocation on the publish hot path.

**Output Files:**
- `Assets/Scripts/Logic/Events/EventBus.cs`

**Acceptance Criteria:**
- Implements `IEventBus` fully
- Type-safe: events are generic `IEvent` structs
- Subscribe returns an `IDisposable` for unsubscription
- Publish has zero heap allocation (pre-allocated handler arrays)
- Thread-safe subscription (concurrent subscribe/unsubscribe safe)
- No LINQ, no reflection in hot paths
- Handles: subscriber added during publish, subscriber removed during publish
- Guard against null handlers
