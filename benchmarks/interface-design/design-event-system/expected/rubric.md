# Rubric: interface-design/design-event-system

## Criteria (1-5 each)

1. **Interface Compliance** — Does it fully implement IEventBus with correct generic constraints?
2. **Zero-Allocation Publish** — Is Publish completely allocation-free? Pre-allocated handler arrays? No LINQ?
3. **Thread Safety** — Are Subscribe/Unsubscribe safe for concurrent access? Lock or copy-on-write?
4. **Edge Cases** — Handles subscribe-during-publish? Unsubscribe-during-publish? Publish with no subscribers?
5. **IDisposable Pattern** — Does Subscribe return a proper IDisposable that unsubscribes on Dispose?
6. **Code Quality** — Sealed class, proper naming, no dead code, clean structure?
