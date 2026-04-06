## Task Assignment

**Task ID:** BENCH-PL-1
**Task Title:** Implement InventorySystem
**Description:** Implement the `InventorySystem` class that fulfills the `IInventorySystem` interface. The system manages a collection of items with add, remove, and query operations. All logic must be pure C# with zero Unity dependencies.

**Output Files:**
- `Assets/Scripts/Logic/Inventory/InventorySystem.cs`

**Acceptance Criteria:**
- Implements `IInventorySystem` fully
- Constructor injection for dependencies (accepts `IInventoryConfig`)
- No `using UnityEngine` statements
- Items stored efficiently with O(1) lookup by ID
- AddItem respects max stack size from config
- RemoveItem handles partial removal
- GetItemCount returns 0 for missing items (no exception)
- Events fired on inventory changes via `IInventoryEvents`
- Zero heap allocation in GetItemCount and HasItem (hot path methods)
