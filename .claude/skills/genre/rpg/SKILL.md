---
name: rpg
description: "RPG game architecture — stat system (base + modifiers), level/XP, skill trees, quest system, NPC interaction, turn-based and real-time combat patterns."
globs: ["**/RPG*.cs", "**/Stat*.cs", "**/Quest*.cs", "**/Skill*.cs", "**/Level*.cs"]
---

# RPG Game Patterns

## Stat System

```csharp
public enum StatModifierType { Flat, PercentAdd, PercentMultiply }

[System.Serializable]
public sealed class StatModifier
{
    public float Value;
    public StatModifierType Type;
    public int Order;
    public object Source;

    public StatModifier(float value, StatModifierType type, int order, object source)
    {
        Value = value;
        Type = type;
        Order = order;
        Source = source;
    }
}

[System.Serializable]
public sealed class CharacterStat
{
    [SerializeField] private float m_BaseValue;
    private readonly List<StatModifier> m_Modifiers = new();
    private float m_CachedValue;
    private bool m_IsDirty = true;

    public float Value
    {
        get
        {
            if (m_IsDirty)
            {
                m_CachedValue = CalculateFinalValue();
                m_IsDirty = false;
            }
            return m_CachedValue;
        }
    }

    public void AddModifier(StatModifier mod)
    {
        m_Modifiers.Add(mod);
        m_Modifiers.Sort((a, b) => a.Order.CompareTo(b.Order));
        m_IsDirty = true;
    }

    public void RemoveAllModifiersFromSource(object source)
    {
        for (int i = m_Modifiers.Count - 1; i >= 0; i--)
        {
            if (m_Modifiers[i].Source == source)
            {
                m_Modifiers.RemoveAt(i);
                m_IsDirty = true;
            }
        }
    }

    private float CalculateFinalValue()
    {
        float finalValue = m_BaseValue;
        float percentAddSum = 0f;

        for (int i = 0; i < m_Modifiers.Count; i++)
        {
            StatModifier mod = m_Modifiers[i];
            switch (mod.Type)
            {
                case StatModifierType.Flat:
                    finalValue += mod.Value;
                    break;
                case StatModifierType.PercentAdd:
                    percentAddSum += mod.Value;
                    if (i + 1 >= m_Modifiers.Count || m_Modifiers[i + 1].Type != StatModifierType.PercentAdd)
                    {
                        finalValue *= 1f + percentAddSum;
                        percentAddSum = 0f;
                    }
                    break;
                case StatModifierType.PercentMultiply:
                    finalValue *= 1f + mod.Value;
                    break;
            }
        }

        return Mathf.Round(finalValue * 100f) / 100f;
    }
}
```

**Modifier ordering:** Flat (+5) → PercentAdd (+10% stacks additively) → PercentMultiply (+20% stacks multiplicatively).

## Level / XP System

```csharp
public sealed class LevelSystem
{
    private int m_CurrentLevel = 1;
    private int m_CurrentXP;
    private int m_MaxLevel = 99;

    public event System.Action<int> OnLevelUp;

    public int CurrentLevel => m_CurrentLevel;
    public int CurrentXP => m_CurrentXP;
    public int XPToNextLevel => GetXPForLevel(m_CurrentLevel + 1) - GetXPForLevel(m_CurrentLevel);
    public float XPProgress => (float)m_CurrentXP / XPToNextLevel;

    public void AddXP(int amount)
    {
        m_CurrentXP += amount;
        while (m_CurrentXP >= XPToNextLevel && m_CurrentLevel < m_MaxLevel)
        {
            m_CurrentXP -= XPToNextLevel;
            m_CurrentLevel++;
            OnLevelUp?.Invoke(m_CurrentLevel);
        }
    }

    // Exponential XP curve
    private int GetXPForLevel(int level)
    {
        return Mathf.RoundToInt(100f * Mathf.Pow(level, 1.5f));
    }
}
```

## Quest System

```csharp
public enum ObjectiveType { Kill, Collect, Talk, Location, Custom }

[System.Serializable]
public sealed class QuestObjective
{
    public string Description;
    public ObjectiveType Type;
    public string TargetId;
    public int RequiredCount;
    [NonSerialized] public int CurrentCount;

    public bool IsComplete => CurrentCount >= RequiredCount;
}

[CreateAssetMenu(menuName = "RPG/Quest Definition")]
public sealed class QuestDefinition : ScriptableObject
{
    [SerializeField] private string m_QuestId;
    [SerializeField] private string m_Title;
    [TextArea] [SerializeField] private string m_Description;
    [SerializeField] private List<QuestObjective> m_Objectives;
    [SerializeField] private int m_XPReward;
    [SerializeField] private List<ItemDefinition> m_ItemRewards;
    [SerializeField] private QuestDefinition[] m_Prerequisites;

    public string QuestId => m_QuestId;
    public string Title => m_Title;
    public IReadOnlyList<QuestObjective> Objectives => m_Objectives;
    public int XPReward => m_XPReward;
}

public sealed class QuestTracker
{
    private readonly Dictionary<string, QuestDefinition> m_ActiveQuests = new();

    public event System.Action<QuestDefinition> OnQuestCompleted;

    public void StartQuest(QuestDefinition quest)
    {
        m_ActiveQuests[quest.QuestId] = quest;
    }

    public void ReportProgress(ObjectiveType type, string targetId, int count = 1)
    {
        foreach (KeyValuePair<string, QuestDefinition> kvp in m_ActiveQuests)
        {
            QuestDefinition quest = kvp.Value;
            bool allComplete = true;

            for (int i = 0; i < quest.Objectives.Count; i++)
            {
                QuestObjective obj = quest.Objectives[i];
                if (obj.Type == type && obj.TargetId == targetId)
                {
                    obj.CurrentCount = Mathf.Min(obj.CurrentCount + count, obj.RequiredCount);
                }
                if (!obj.IsComplete) allComplete = false;
            }

            if (allComplete)
            {
                OnQuestCompleted?.Invoke(quest);
            }
        }
    }
}
```

## Skill Tree

- **SkillNode SO:** name, description, icon, prerequisites (list of SkillNode), stat modifiers, unlock cost
- **SkillTree:** graph of nodes, check prerequisites before unlock, apply modifiers on unlock
- **Respec:** remove all modifiers from skill source, reset unlock state

## Combat Patterns

### Real-Time (Action RPG)
- Abilities as ScriptableObjects: damage, cooldown, mana cost, animation trigger, VFX
- Combo system: track input sequence within time window
- Status effects: timed stat modifiers + tick damage/heal + VFX

### Turn-Based
- Turn order by Speed stat (or initiative roll)
- Action queue: select action → select target → execute → next turn
- Command pattern: each action is an ICommand with Execute/Undo
