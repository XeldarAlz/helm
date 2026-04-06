# Rubric: pure-logic/implement-inventory

## Criteria (1-5 each)

1. **Interface Compliance** — Does it fully implement IInventorySystem? All methods present with correct signatures?
2. **Constructor Injection** — Are IInventoryConfig and IInventoryEvents injected via constructor? No static access?
3. **Zero-Allocation Hot Paths** — Are GetItemCount and HasItem allocation-free? No LINQ, no new, no boxing?
4. **Naming Conventions** — PascalCase types/methods, _camelCase private fields, camelCase params?
5. **Edge Case Handling** — Does AddItem respect MaxStackSize? Does RemoveItem handle partial? Does GetItemCount return 0 for missing?
6. **Code Quality** — Clean structure, guard clauses, no dead code, no XML docs, appropriate access modifiers?
