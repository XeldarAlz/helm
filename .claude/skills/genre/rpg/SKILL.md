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

---

## Equipment System

### Equipment Slots and Definitions

```csharp
public enum EquipmentSlot
{
    Weapon,
    Armor,
    Helmet,
    Boots,
    Accessory,
    Ring
}

public enum RarityTier
{
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary
}

[CreateAssetMenu(menuName = "RPG/Equipment Definition")]
public sealed class EquipmentDefinition : ScriptableObject
{
    [SerializeField] private string m_EquipmentId;
    [SerializeField] private string m_DisplayName;
    [SerializeField] private Sprite m_Icon;
    [SerializeField] private EquipmentSlot m_Slot;
    [SerializeField] private RarityTier m_Rarity;
    [SerializeField] private int m_RequiredLevel;
    [SerializeField] private List<StatBonus> m_StatBonuses;

    public string EquipmentId => m_EquipmentId;
    public string DisplayName => m_DisplayName;
    public Sprite Icon => m_Icon;
    public EquipmentSlot Slot => m_Slot;
    public RarityTier Rarity => m_Rarity;
    public int RequiredLevel => m_RequiredLevel;
    public IReadOnlyList<StatBonus> StatBonuses => m_StatBonuses;
}

[System.Serializable]
public sealed class StatBonus
{
    [SerializeField] private StatType m_StatType;
    [SerializeField] private StatModifierType m_ModifierType;
    [SerializeField] private float m_Value;

    public StatType StatType => m_StatType;
    public StatModifierType ModifierType => m_ModifierType;
    public float Value => m_Value;
}
```

### Equipment System (Plain C#, VContainer-Injected)

```csharp
public readonly struct EquipmentChangedMessage
{
    public readonly EquipmentSlot Slot;
    public readonly EquipmentDefinition OldItem;
    public readonly EquipmentDefinition NewItem;

    public EquipmentChangedMessage(EquipmentSlot slot, EquipmentDefinition oldItem, EquipmentDefinition newItem)
    {
        Slot = slot;
        OldItem = oldItem;
        NewItem = newItem;
    }
}

public sealed class EquipmentSystem : IDisposable
{
    private readonly EquipmentModel m_Model;
    private readonly CharacterStatsModel m_Stats;
    private readonly IPublisher<EquipmentChangedMessage> m_EquipChangedPublisher;

    [Inject]
    public EquipmentSystem(
        EquipmentModel model,
        CharacterStatsModel stats,
        IPublisher<EquipmentChangedMessage> equipChangedPublisher)
    {
        m_Model = model;
        m_Stats = stats;
        m_EquipChangedPublisher = equipChangedPublisher;
    }

    public bool TryEquip(EquipmentDefinition equipment)
    {
        if (m_Stats.Level < equipment.RequiredLevel) return false;

        EquipmentDefinition oldItem = m_Model.GetEquipped(equipment.Slot);
        if (oldItem != null)
        {
            RemoveStatModifiers(oldItem);
        }

        m_Model.SetEquipped(equipment.Slot, equipment);
        ApplyStatModifiers(equipment);
        m_EquipChangedPublisher.Publish(new EquipmentChangedMessage(equipment.Slot, oldItem, equipment));
        return true;
    }

    public void Unequip(EquipmentSlot slot)
    {
        EquipmentDefinition oldItem = m_Model.GetEquipped(slot);
        if (oldItem == null) return;

        RemoveStatModifiers(oldItem);
        m_Model.SetEquipped(slot, null);
        m_EquipChangedPublisher.Publish(new EquipmentChangedMessage(slot, oldItem, null));
    }

    private void ApplyStatModifiers(EquipmentDefinition equipment)
    {
        for (int bonusIndex = 0; bonusIndex < equipment.StatBonuses.Count; bonusIndex++)
        {
            StatBonus bonus = equipment.StatBonuses[bonusIndex];
            int order = bonus.ModifierType switch
            {
                StatModifierType.Flat => 100,
                StatModifierType.PercentAdd => 200,
                StatModifierType.PercentMultiply => 300,
                _ => 100
            };
            CharacterStat stat = m_Stats.GetStat(bonus.StatType);
            stat.AddModifier(new StatModifier(bonus.Value, bonus.ModifierType, order, equipment));
        }
    }

    private void RemoveStatModifiers(EquipmentDefinition equipment)
    {
        for (int bonusIndex = 0; bonusIndex < equipment.StatBonuses.Count; bonusIndex++)
        {
            StatBonus bonus = equipment.StatBonuses[bonusIndex];
            CharacterStat stat = m_Stats.GetStat(bonus.StatType);
            stat.RemoveAllModifiersFromSource(equipment);
        }
    }

    public void Dispose() { }
}
```

Always recalculate all stats after equip/unequip by removing then re-adding modifiers. Never cache partial results across equipment changes.

---

## Combat Abilities

### Ability Definition

```csharp
public enum AbilityEffectType
{
    Damage,
    Heal,
    Buff,
    Debuff,
    AreaDamage,
    Projectile
}

[CreateAssetMenu(menuName = "RPG/Ability Definition")]
public sealed class AbilityDefinition : ScriptableObject
{
    [SerializeField] private string m_AbilityId;
    [SerializeField] private string m_DisplayName;
    [SerializeField] private Sprite m_Icon;
    [SerializeField] private float m_Cooldown = 1f;
    [SerializeField] private int m_ManaCost;
    [SerializeField] private float m_Range = 5f;
    [SerializeField] private float m_BaseDamage;
    [SerializeField] private AbilityEffectType m_EffectType;
    [SerializeField] private float m_CastTime;
    [SerializeField] private float m_VfxDelay;
    [SerializeField] private GameObject m_VfxPrefab;
    [SerializeField] private string m_AnimationTrigger;

    public string AbilityId => m_AbilityId;
    public string DisplayName => m_DisplayName;
    public float Cooldown => m_Cooldown;
    public int ManaCost => m_ManaCost;
    public float Range => m_Range;
    public float BaseDamage => m_BaseDamage;
    public AbilityEffectType EffectType => m_EffectType;
    public float CastTime => m_CastTime;
    public float VfxDelay => m_VfxDelay;
    public GameObject VfxPrefab => m_VfxPrefab;
    public string AnimationTrigger => m_AnimationTrigger;
}
```

### Ability Cast System

```csharp
public readonly struct AbilityCastMessage
{
    public readonly AbilityDefinition Ability;
    public readonly Vector3 TargetPosition;

    public AbilityCastMessage(AbilityDefinition ability, Vector3 targetPosition)
    {
        Ability = ability;
        TargetPosition = targetPosition;
    }
}

public sealed class AbilityCastSystem : IDisposable
{
    private readonly Dictionary<string, float> m_CooldownTimers = new();
    private readonly CancellationTokenSource m_Cts = new();
    private readonly CharacterStatsModel m_Stats;
    private readonly IPublisher<AbilityCastMessage> m_CastPublisher;

    // Ability queue: buffer next ability during current cast animation
    private AbilityDefinition m_QueuedAbility;
    private Vector3 m_QueuedTarget;
    private bool m_IsCasting;

    [Inject]
    public AbilityCastSystem(
        CharacterStatsModel stats,
        IPublisher<AbilityCastMessage> castPublisher)
    {
        m_Stats = stats;
        m_CastPublisher = castPublisher;
    }

    public bool TryCastAbility(AbilityDefinition ability, Vector3 casterPos, Vector3 targetPos)
    {
        if (m_IsCasting)
        {
            // Buffer the ability for execution after current cast completes
            m_QueuedAbility = ability;
            m_QueuedTarget = targetPos;
            return true;
        }

        if (!CanCast(ability, casterPos, targetPos)) return false;

        CastAbilityAsync(ability, targetPos).Forget();
        return true;
    }

    private bool CanCast(AbilityDefinition ability, Vector3 casterPos, Vector3 targetPos)
    {
        // Check cooldown
        if (m_CooldownTimers.TryGetValue(ability.AbilityId, out float cooldownEnd))
        {
            if (Time.time < cooldownEnd) return false;
        }

        // Check mana
        if (m_Stats.CurrentMana < ability.ManaCost) return false;

        // Check range
        float distanceSqr = (targetPos - casterPos).sqrMagnitude;
        if (distanceSqr > ability.Range * ability.Range) return false;

        return true;
    }

    private async UniTaskVoid CastAbilityAsync(AbilityDefinition ability, Vector3 targetPos)
    {
        m_IsCasting = true;
        m_Stats.CurrentMana -= ability.ManaCost;
        m_CooldownTimers[ability.AbilityId] = Time.time + ability.Cooldown;

        // Cast time delay (animation plays during this)
        if (ability.CastTime > 0f)
        {
            await UniTask.Delay(
                System.TimeSpan.FromSeconds(ability.CastTime),
                cancellationToken: m_Cts.Token);
        }

        // VFX timing offset — fire the VFX at the right moment in the animation
        if (ability.VfxDelay > 0f)
        {
            await UniTask.Delay(
                System.TimeSpan.FromSeconds(ability.VfxDelay),
                cancellationToken: m_Cts.Token);
        }

        m_CastPublisher.Publish(new AbilityCastMessage(ability, targetPos));
        m_IsCasting = false;

        // Execute queued ability if one was buffered
        if (m_QueuedAbility != null)
        {
            AbilityDefinition queued = m_QueuedAbility;
            Vector3 queuedTarget = m_QueuedTarget;
            m_QueuedAbility = null;
            TryCastAbility(queued, Vector3.zero, queuedTarget);
        }
    }

    public void UpdateCooldowns(float deltaTime)
    {
        // Called from game loop tick — no per-frame allocation needed
        // Cooldowns are stored as absolute end times, so no update logic is required
    }

    public float GetCooldownRemaining(AbilityDefinition ability)
    {
        if (m_CooldownTimers.TryGetValue(ability.AbilityId, out float cooldownEnd))
        {
            return Mathf.Max(0f, cooldownEnd - Time.time);
        }
        return 0f;
    }

    public void Dispose() => m_Cts.Cancel();
}
```

The ability queue pattern ensures smooth chaining: the player can press the next ability during the current cast animation, and it fires immediately when the current cast completes. Only one ability is buffered at a time to prevent unintentional spam.

---

## Status Effects Framework

### Status Effect Definition and Data

```csharp
public enum StackingBehavior
{
    None,           // New application refreshes duration only
    Duration,       // Extends duration, same intensity
    Intensity,      // Stacks intensity up to a cap, same duration
    Independent     // Each application tracked separately
}

[CreateAssetMenu(menuName = "RPG/Status Effect Definition")]
public sealed class StatusEffectDefinition : ScriptableObject
{
    [SerializeField] private string m_EffectId;
    [SerializeField] private string m_DisplayName;
    [SerializeField] private Sprite m_Icon;
    [SerializeField] private float m_Duration = 5f;
    [SerializeField] private float m_TickInterval = 1f;
    [SerializeField] private float m_ValuePerTick;
    [SerializeField] private StatType m_AffectedStat;
    [SerializeField] private StatModifierType m_ModifierType;
    [SerializeField] private float m_StatModValue;
    [SerializeField] private StackingBehavior m_Stacking;
    [SerializeField] private int m_MaxStacks = 1;

    public string EffectId => m_EffectId;
    public string DisplayName => m_DisplayName;
    public Sprite Icon => m_Icon;
    public float Duration => m_Duration;
    public float TickInterval => m_TickInterval;
    public float ValuePerTick => m_ValuePerTick;
    public StatType AffectedStat => m_AffectedStat;
    public StatModifierType ModifierType => m_ModifierType;
    public float StatModValue => m_StatModValue;
    public StackingBehavior Stacking => m_Stacking;
    public int MaxStacks => m_MaxStacks;
}
```

### Active Effect Runtime Data

```csharp
public sealed class ActiveStatusEffect
{
    public StatusEffectDefinition Definition;
    public float RemainingDuration;
    public float TickTimer;
    public int CurrentStacks;
    public StatModifier AppliedModifier;
}
```

### Status Effect System

```csharp
public readonly struct StatusEffectAppliedMessage
{
    public readonly int EntityId;
    public readonly StatusEffectDefinition Effect;
    public readonly int Stacks;

    public StatusEffectAppliedMessage(int entityId, StatusEffectDefinition effect, int stacks)
    {
        EntityId = entityId;
        Effect = effect;
        Stacks = stacks;
    }
}

public readonly struct StatusEffectRemovedMessage
{
    public readonly int EntityId;
    public readonly StatusEffectDefinition Effect;

    public StatusEffectRemovedMessage(int entityId, StatusEffectDefinition effect)
    {
        EntityId = entityId;
        Effect = effect;
    }
}

public sealed class StatusEffectSystem : IDisposable
{
    // Per-entity active effects: entityId → list of active effects
    private readonly Dictionary<int, List<ActiveStatusEffect>> m_ActiveEffects = new();
    private readonly IPublisher<StatusEffectAppliedMessage> m_AppliedPublisher;
    private readonly IPublisher<StatusEffectRemovedMessage> m_RemovedPublisher;

    [Inject]
    public StatusEffectSystem(
        IPublisher<StatusEffectAppliedMessage> appliedPublisher,
        IPublisher<StatusEffectRemovedMessage> removedPublisher)
    {
        m_AppliedPublisher = appliedPublisher;
        m_RemovedPublisher = removedPublisher;
    }

    public void ApplyEffect(int entityId, StatusEffectDefinition definition, CharacterStatsModel stats)
    {
        if (!m_ActiveEffects.TryGetValue(entityId, out List<ActiveStatusEffect> effects))
        {
            effects = new List<ActiveStatusEffect>(8);
            m_ActiveEffects[entityId] = effects;
        }

        // Check for existing effect of the same type
        ActiveStatusEffect existing = null;
        for (int effectIndex = 0; effectIndex < effects.Count; effectIndex++)
        {
            if (effects[effectIndex].Definition.EffectId == definition.EffectId)
            {
                existing = effects[effectIndex];
                break;
            }
        }

        if (existing != null)
        {
            switch (definition.Stacking)
            {
                case StackingBehavior.None:
                    existing.RemainingDuration = definition.Duration;
                    return;
                case StackingBehavior.Duration:
                    existing.RemainingDuration += definition.Duration;
                    return;
                case StackingBehavior.Intensity:
                    if (existing.CurrentStacks < definition.MaxStacks)
                    {
                        existing.CurrentStacks++;
                        existing.RemainingDuration = definition.Duration;
                        UpdateStackModifier(existing, stats);
                    }
                    m_AppliedPublisher.Publish(new StatusEffectAppliedMessage(entityId, definition, existing.CurrentStacks));
                    return;
                case StackingBehavior.Independent:
                    break; // Fall through to create new instance
            }
        }

        var newEffect = new ActiveStatusEffect
        {
            Definition = definition,
            RemainingDuration = definition.Duration,
            TickTimer = definition.TickInterval,
            CurrentStacks = 1
        };

        // Apply stat modifier if the effect modifies a stat (e.g., Slow reduces speed)
        if (definition.StatModValue != 0f)
        {
            var modifier = new StatModifier(definition.StatModValue, definition.ModifierType, 400, definition);
            stats.GetStat(definition.AffectedStat).AddModifier(modifier);
            newEffect.AppliedModifier = modifier;
        }

        effects.Add(newEffect);
        m_AppliedPublisher.Publish(new StatusEffectAppliedMessage(entityId, definition, 1));
    }

    // Call once per game tick from the game loop entry point
    public void Tick(float deltaTime, Dictionary<int, CharacterStatsModel> entityStats)
    {
        foreach (KeyValuePair<int, List<ActiveStatusEffect>> kvp in m_ActiveEffects)
        {
            int entityId = kvp.Key;
            List<ActiveStatusEffect> effects = kvp.Value;
            if (!entityStats.TryGetValue(entityId, out CharacterStatsModel stats)) continue;

            for (int effectIndex = effects.Count - 1; effectIndex >= 0; effectIndex--)
            {
                ActiveStatusEffect effect = effects[effectIndex];
                effect.RemainingDuration -= deltaTime;

                // Tick damage/heal (e.g., Poison deals ValuePerTick every TickInterval)
                effect.TickTimer -= deltaTime;
                if (effect.TickTimer <= 0f)
                {
                    effect.TickTimer += effect.Definition.TickInterval;
                    ApplyTickEffect(entityId, effect, stats);
                }

                // Remove expired effects
                if (effect.RemainingDuration <= 0f)
                {
                    RemoveEffect(entityId, effect, stats);
                    effects.RemoveAt(effectIndex);
                }
            }
        }
    }

    private void ApplyTickEffect(int entityId, ActiveStatusEffect effect, CharacterStatsModel stats)
    {
        float tickValue = effect.Definition.ValuePerTick * effect.CurrentStacks;
        // Positive = heal, Negative = damage (e.g., Poison has negative ValuePerTick)
        stats.CurrentHealth += Mathf.RoundToInt(tickValue);
    }

    private void UpdateStackModifier(ActiveStatusEffect effect, CharacterStatsModel stats)
    {
        if (effect.AppliedModifier != null)
        {
            stats.GetStat(effect.Definition.AffectedStat).RemoveAllModifiersFromSource(effect.Definition);
        }
        float stackedValue = effect.Definition.StatModValue * effect.CurrentStacks;
        var modifier = new StatModifier(stackedValue, effect.Definition.ModifierType, 400, effect.Definition);
        stats.GetStat(effect.Definition.AffectedStat).AddModifier(modifier);
        effect.AppliedModifier = modifier;
    }

    private void RemoveEffect(int entityId, ActiveStatusEffect effect, CharacterStatsModel stats)
    {
        if (effect.AppliedModifier != null)
        {
            stats.GetStat(effect.Definition.AffectedStat).RemoveAllModifiersFromSource(effect.Definition);
        }
        m_RemovedPublisher.Publish(new StatusEffectRemovedMessage(entityId, effect.Definition));
    }

    public void Dispose() { }
}
```

### Common Status Effect Examples

- **Poison:** `ValuePerTick = -5`, `TickInterval = 1`, `Duration = 8`, `Stacking = Intensity`, `MaxStacks = 3`
- **Slow:** `AffectedStat = MoveSpeed`, `StatModValue = -0.3f`, `ModifierType = PercentMultiply`, `Duration = 4`
- **Shield:** `AffectedStat = Defense`, `StatModValue = 50`, `ModifierType = Flat`, `Duration = 10`
- **Regeneration:** `ValuePerTick = 3`, `TickInterval = 0.5f`, `Duration = 12`, `Stacking = Duration`

The StatusEffectView subscribes to `StatusEffectAppliedMessage` and `StatusEffectRemovedMessage` via MessagePipe to show/hide buff icons and duration bars. The View never ticks or manages effect logic.

---

## Dialogue Integration

### NPC Interaction Trigger

```csharp
public sealed class NPCInteractionView : MonoBehaviour
{
    [SerializeField] private float m_InteractionRange = 2.5f;
    [SerializeField] private string m_NpcId;

    private Transform m_PlayerTransform;
    private NPCInteractionSystem m_InteractionSystem;
    private bool m_IsInRange;

    [Inject]
    public void Construct(NPCInteractionSystem interactionSystem)
    {
        m_InteractionSystem = interactionSystem;
    }

    // Called by the input handler when interact button is pressed
    public void OnInteractInput()
    {
        if (!m_IsInRange) return;
        m_InteractionSystem.StartInteraction(m_NpcId);
    }

    // Ground check via pre-cached player reference, updated by parent system
    public void UpdateProximity(Transform playerTransform)
    {
        m_PlayerTransform = playerTransform;
        if (m_PlayerTransform == null) return;
        float distanceSqr = (m_PlayerTransform.position - transform.position).sqrMagnitude;
        m_IsInRange = distanceSqr <= m_InteractionRange * m_InteractionRange;
    }
}
```

### Dialogue with Quest-State Branching

```csharp
public sealed class NPCInteractionSystem : IDisposable
{
    private readonly QuestTracker m_QuestTracker;
    private readonly IPublisher<DialogueStartedMessage> m_DialoguePublisher;
    private readonly IPublisher<ShopOpenedMessage> m_ShopPublisher;
    private readonly Dictionary<string, NPCDialogueData> m_NpcData;

    [Inject]
    public NPCInteractionSystem(
        QuestTracker questTracker,
        IPublisher<DialogueStartedMessage> dialoguePublisher,
        IPublisher<ShopOpenedMessage> shopPublisher)
    {
        m_QuestTracker = questTracker;
        m_DialoguePublisher = dialoguePublisher;
        m_ShopPublisher = shopPublisher;
        m_NpcData = new Dictionary<string, NPCDialogueData>();
    }

    public void StartInteraction(string npcId)
    {
        if (!m_NpcData.TryGetValue(npcId, out NPCDialogueData data)) return;

        // Select dialogue branch based on quest state
        string dialogueKey = data.DefaultDialogueKey;
        for (int branchIndex = 0; branchIndex < data.QuestBranches.Count; branchIndex++)
        {
            QuestDialogueBranch branch = data.QuestBranches[branchIndex];
            if (m_QuestTracker.IsQuestInState(branch.QuestId, branch.RequiredState))
            {
                dialogueKey = branch.DialogueKey;
                break; // First matching branch wins (priority order)
            }
        }

        m_DialoguePublisher.Publish(new DialogueStartedMessage(npcId, dialogueKey));
    }

    // Called when dialogue completes — handles reward granting and shop opening
    public void OnDialogueComplete(string npcId, string outcomeId)
    {
        if (!m_NpcData.TryGetValue(npcId, out NPCDialogueData data)) return;

        if (data.GrantsRewardOnComplete)
        {
            // Grant quest rewards, XP, items via respective systems
        }

        if (data.OpensShopOnComplete)
        {
            m_ShopPublisher.Publish(new ShopOpenedMessage(data.ShopInventoryId));
        }
    }

    public void Dispose() { }
}
```

The shop dialogue flow opens an inventory comparison view: the player sees shop items alongside their current equipment, with stat differences highlighted (green for upgrades, red for downgrades).

---

## Party System

### Party Model

```csharp
public sealed class PartyModel
{
    private readonly List<PartyMemberData> m_ActiveMembers = new(4);
    private readonly List<PartyMemberData> m_ReserveMembers = new(8);
    private int m_MaxActiveSize = 4;

    public IReadOnlyList<PartyMemberData> ActiveMembers => m_ActiveMembers;
    public IReadOnlyList<PartyMemberData> ReserveMembers => m_ReserveMembers;

    public bool TryAddMember(PartyMemberData member)
    {
        if (m_ActiveMembers.Count < m_MaxActiveSize)
        {
            m_ActiveMembers.Add(member);
            return true;
        }
        m_ReserveMembers.Add(member);
        return false;
    }

    public bool SwapMember(int activeIndex, int reserveIndex)
    {
        if (activeIndex < 0 || activeIndex >= m_ActiveMembers.Count) return false;
        if (reserveIndex < 0 || reserveIndex >= m_ReserveMembers.Count) return false;

        PartyMemberData temp = m_ActiveMembers[activeIndex];
        m_ActiveMembers[activeIndex] = m_ReserveMembers[reserveIndex];
        m_ReserveMembers[reserveIndex] = temp;
        return true;
    }
}

public sealed class PartyMemberData
{
    public string CharacterId;
    public CharacterStatsModel Stats;
    public EquipmentModel Equipment;
    public List<AbilityDefinition> LearnedAbilities;
}
```

### Party System with Buffs

```csharp
public readonly struct PartyBuffAppliedMessage
{
    public readonly StatusEffectDefinition Buff;
    public readonly int MemberCount;

    public PartyBuffAppliedMessage(StatusEffectDefinition buff, int memberCount)
    {
        Buff = buff;
        MemberCount = memberCount;
    }
}

public sealed class PartySystem : IDisposable
{
    private readonly PartyModel m_Model;
    private readonly StatusEffectSystem m_StatusEffects;
    private readonly IPublisher<PartyBuffAppliedMessage> m_BuffPublisher;

    [Inject]
    public PartySystem(
        PartyModel model,
        StatusEffectSystem statusEffects,
        IPublisher<PartyBuffAppliedMessage> buffPublisher)
    {
        m_Model = model;
        m_StatusEffects = statusEffects;
        m_BuffPublisher = buffPublisher;
    }

    // Apply a buff to all active party members
    public void ApplyPartyBuff(StatusEffectDefinition buff)
    {
        for (int memberIndex = 0; memberIndex < m_Model.ActiveMembers.Count; memberIndex++)
        {
            PartyMemberData member = m_Model.ActiveMembers[memberIndex];
            m_StatusEffects.ApplyEffect(member.CharacterId.GetHashCode(), buff, member.Stats);
        }
        m_BuffPublisher.Publish(new PartyBuffAppliedMessage(buff, m_Model.ActiveMembers.Count));
    }

    public void SwapActiveAndReserve(int activeIndex, int reserveIndex)
    {
        m_Model.SwapMember(activeIndex, reserveIndex);
        // Re-apply any party-wide aura effects to the newly active member
    }

    public void Dispose() { }
}
```

**Shared vs individual inventories:** Use a single `InventoryModel` for party-wide items (consumables, key items) and per-member `EquipmentModel` for gear. The party inventory is accessible by all members; equipment is bound to a specific character.

**Formation:** Store formation as `Vector2[]` offsets from the party leader. The movement system applies these offsets with a follow delay for natural-looking group movement.

---

## Save System Integration

### Serializable Character State

```csharp
[System.Serializable]
public sealed class CharacterSaveData
{
    public string CharacterId;
    public int Level;
    public int CurrentXP;
    public int CurrentHealth;
    public int CurrentMana;
    public float PositionX;
    public float PositionY;
    public float PositionZ;
    public List<EquipmentSaveEntry> EquippedItems;
    public List<string> LearnedAbilityIds;
    public List<StatusEffectSaveEntry> ActiveEffects;
}

[System.Serializable]
public sealed class EquipmentSaveEntry
{
    public EquipmentSlot Slot;
    public string EquipmentId;
}

[System.Serializable]
public sealed class StatusEffectSaveEntry
{
    public string EffectId;
    public float RemainingDuration;
    public int CurrentStacks;
}

[System.Serializable]
public sealed class InventorySaveData
{
    public List<ItemSaveEntry> Items;
}

[System.Serializable]
public sealed class ItemSaveEntry
{
    public string ItemId;
    public int Count;
}
```

### Quest Progress Persistence

```csharp
[System.Serializable]
public sealed class QuestSaveData
{
    public List<QuestProgressEntry> ActiveQuests;
    public List<string> CompletedQuestIds;
}

[System.Serializable]
public sealed class QuestProgressEntry
{
    public string QuestId;
    public List<int> ObjectiveProgress; // CurrentCount per objective index
}
```

### Save/Load with Validation

```csharp
public sealed class RPGSaveSystem
{
    private readonly Dictionary<string, EquipmentDefinition> m_EquipmentLookup;
    private readonly Dictionary<string, AbilityDefinition> m_AbilityLookup;
    private readonly Dictionary<string, StatusEffectDefinition> m_EffectLookup;

    public CharacterSaveData CreateSaveData(
        string characterId,
        CharacterStatsModel stats,
        EquipmentModel equipment,
        Vector3 position)
    {
        var save = new CharacterSaveData
        {
            CharacterId = characterId,
            Level = stats.Level,
            CurrentXP = stats.CurrentXP,
            CurrentHealth = stats.CurrentHealth,
            CurrentMana = stats.CurrentMana,
            PositionX = position.x,
            PositionY = position.y,
            PositionZ = position.z,
            EquippedItems = new List<EquipmentSaveEntry>(),
            LearnedAbilityIds = new List<string>(),
            ActiveEffects = new List<StatusEffectSaveEntry>()
        };

        // Serialize equipped items by ID only — never serialize the SO reference
        foreach (EquipmentSlot slot in System.Enum.GetValues(typeof(EquipmentSlot)))
        {
            EquipmentDefinition equipped = equipment.GetEquipped(slot);
            if (equipped != null)
            {
                save.EquippedItems.Add(new EquipmentSaveEntry { Slot = slot, EquipmentId = equipped.EquipmentId });
            }
        }

        return save;
    }

    public void LoadAndValidate(CharacterSaveData save, CharacterStatsModel stats, EquipmentSystem equipment)
    {
        stats.Level = save.Level;
        stats.CurrentXP = save.CurrentXP;
        stats.CurrentHealth = Mathf.Min(save.CurrentHealth, stats.MaxHealth);
        stats.CurrentMana = Mathf.Min(save.CurrentMana, stats.MaxMana);

        // Validate equipment: skip items that no longer exist in the database
        for (int entryIndex = 0; entryIndex < save.EquippedItems.Count; entryIndex++)
        {
            EquipmentSaveEntry entry = save.EquippedItems[entryIndex];
            if (m_EquipmentLookup.TryGetValue(entry.EquipmentId, out EquipmentDefinition definition))
            {
                equipment.TryEquip(definition);
            }
            // Silently skip removed equipment — log warning in editor
        }

        // Validate abilities: skip abilities removed in patches
        for (int abilityIndex = 0; abilityIndex < save.LearnedAbilityIds.Count; abilityIndex++)
        {
            string abilityId = save.LearnedAbilityIds[abilityIndex];
            if (!m_AbilityLookup.ContainsKey(abilityId))
            {
                // Ability was removed — skip, do not crash
            }
        }
    }
}
```

Save by ID, load by lookup. Never serialize ScriptableObject references directly in save files — they break when assets are renamed or moved.

---

## Common Pitfalls

### Stat Modifier Ordering
Always apply modifiers in a consistent order: Flat (order 100) → PercentAdd (order 200) → PercentMultiply (order 300). If equipment and status effects both add PercentAdd modifiers, they must share the same order value so they stack additively with each other rather than multiplicatively. The `CharacterStat.CalculateFinalValue` method handles this automatically when orders are set correctly.

### Equipment Duplication on Save/Load
Never store equipment in both the inventory and the equipment model simultaneously. When equipping, remove from inventory first, then add to equipment. When saving, serialize equipped item IDs separately from inventory item IDs. On load, rebuild the equipment state by looking up definitions from IDs — do not clone or instantiate ScriptableObjects.

### Status Effect Stacking Overflow
Cap stacks with `MaxStacks` on the definition. Without a cap, stackable effects like Poison can multiply to absurd damage values. When stacks reach the max, refresh duration but do not increase intensity. Always validate the stack count before multiplying the effect value.

### Dialogue State Desync After Load
Quest state and dialogue state must be loaded together. If the quest tracker loads progress but the NPC dialogue system still has stale cached data, the player sees wrong dialogue branches. Clear all NPC dialogue caches on load and re-evaluate branches from the quest tracker state. Never cache dialogue branch selections across save/load boundaries.

### Cooldown Timer Persistence
If the game saves while abilities are on cooldown, store the remaining cooldown time in the save data. On load, restore cooldowns relative to the current time. Alternatively, reset all cooldowns on load for a cleaner player experience — pick one approach and apply it consistently.

### Stat Recalculation After Load
After loading equipment and status effects, force a full stat recalculation by marking all `CharacterStat` instances as dirty. Loading equipment triggers `AddModifier` which handles this, but if any path skips the normal equip flow, stats will be stale until the next modification.
