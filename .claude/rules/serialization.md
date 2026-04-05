# Serialization Rules

## CRITICAL: FormerlySerializedAs

When renaming ANY serialized field, you MUST add `[FormerlySerializedAs]`:

```csharp
// Renaming m_Speed to m_MoveSpeed:
[FormerlySerializedAs("m_Speed")]
[SerializeField] private float m_MoveSpeed;
```

**Why:** Without this, Unity cannot map the old serialized data to the new field name. Every configured value in every scene, prefab, and ScriptableObject silently resets to default. Hours of artist/designer work lost with zero warning.

The `[FormerlySerializedAs]` attribute stays forever. Never remove it.

## What Unity Serializes

**Serialized:**
- `public` fields (without `[NonSerialized]`)
- `[SerializeField] private` fields
- Supported types: primitives, string, Vector2/3/4, Color, Rect, AnimationCurve, enums, arrays, List<T>, UnityEngine.Object references

**NOT Serialized:**
- Properties (even with `[SerializeField]`)
- `static` fields
- `readonly` fields
- `const` fields
- `Dictionary<K,V>` (use `ISerializationCallbackReceiver`)
- Interfaces/abstract types (use `[SerializeReference]`)

## Field Exposure

```csharp
// GOOD — private with explicit serialization
[SerializeField] private float m_Health = 100f;

// BAD — public exposes to inspector AND code
public float health = 100f;

// For auto-properties (C# 7.3+):
[field: SerializeField] public float Health { get; private set; }
```

- `[HideInInspector]` — hides from inspector but STILL serializes
- `[NonSerialized]` — prevents serialization entirely (for cached/computed data on public fields)

## Unity Null Check

```csharp
// CORRECT — Unity overrides == to detect destroyed objects
if (m_Target == null) return;

// WRONG — bypasses Unity's destroyed-object detection
if (m_Target is null) return;      // C# null check, misses destroyed
if (m_Target?.Method() != null)    // ?. bypasses Unity's ==, calls method on destroyed object
```

Unity objects can be "destroyed" but not garbage collected. `== null` returns true for destroyed objects. `?.` and `is null` use C# reference equality, which returns false — leading to calls on destroyed objects.

## Polymorphic Serialization

```csharp
// For interface/abstract fields:
[SerializeReference] private IAbility m_Ability;
```

Without `[SerializeReference]`, Unity serializes by value and loses type information.

## Custom Types

```csharp
// Dictionary serialization via callback:
public class MyData : MonoBehaviour, ISerializationCallbackReceiver
{
    [SerializeField] private List<string> m_Keys = new();
    [SerializeField] private List<int> m_Values = new();

    private Dictionary<string, int> m_Lookup = new();

    public void OnBeforeSerialize()
    {
        m_Keys.Clear();
        m_Values.Clear();
        foreach (KeyValuePair<string, int> pair in m_Lookup)
        {
            m_Keys.Add(pair.Key);
            m_Values.Add(pair.Value);
        }
    }

    public void OnAfterDeserialize()
    {
        m_Lookup.Clear();
        for (int i = 0; i < m_Keys.Count; i++)
        {
            m_Lookup[m_Keys[i]] = m_Values[i];
        }
    }
}
```

## Depth Limit

Unity serialization stops at 7 levels of nesting. Deeply nested data structures will be silently truncated.

## Prefab Overrides

Changing a serialized field on a prefab instance creates a prefab override. `[FormerlySerializedAs]` preserves these overrides during renames. Without it, all overrides are lost.
