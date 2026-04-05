---
name: Card Game
description: Card game patterns — deck building, hand management, turn structure, card effects, battlefield zones
globs: ["**/Card*.cs", "**/Deck*.cs", "**/Hand*.cs", "**/Discard*.cs", "**/Mana*.cs", "**/Spell*.cs"]
---

# Card Game Patterns

Comprehensive reference for building card games in Unity. Covers deck construction, hand management, zone tracking, turn flow, effect resolution, and UI layout. Applies to TCGs, deck-builders, and hybrid card-battlers.

## Overview

The core loop of a card game:

1. **Draw Phase** -- player draws cards from deck into hand
2. **Resource Phase** -- mana/energy refreshes or is gained
3. **Main Phase** -- play cards from hand by spending resources
4. **Combat Phase** -- minions/creatures attack (if applicable)
5. **End Phase** -- end-of-turn triggers fire, pass to opponent

Each card has a cost, a type, and one or more effects. Cards move between zones (deck, hand, battlefield, graveyard). The game ends when a win condition is met (opponent health reaches zero, deck-out, etc.).

All game logic lives in plain C# Systems. MonoBehaviour Views observe Models and render the visual state. VContainer wires everything. MessagePipe carries zone-transfer and effect events.

---

## Card Data Model

### CardDefinition (ScriptableObject)

Static card data. Never mutated at runtime.

```csharp
public enum CardType { Minion, Spell, Trap, Equipment }
public enum CardRarity { Common, Uncommon, Rare, Legendary }

[CreateAssetMenu(menuName = "CardGame/Card Definition")]
public sealed class CardDefinition : ScriptableObject
{
    [SerializeField] private string m_CardId;
    [SerializeField] private string m_DisplayName;
    [TextArea] [SerializeField] private string m_Description;
    [SerializeField] private Sprite m_Artwork;
    [SerializeField] private CardType m_Type;
    [SerializeField] private CardRarity m_Rarity;
    [SerializeField] private int m_ManaCost;
    [SerializeField] private int m_Attack;
    [SerializeField] private int m_Health;
    [SerializeField] private EffectDefinition[] m_Effects;
    [SerializeField] private string[] m_Tags;

    public string CardId => m_CardId;
    public string DisplayName => m_DisplayName;
    public string Description => m_Description;
    public Sprite Artwork => m_Artwork;
    public CardType Type => m_Type;
    public CardRarity Rarity => m_Rarity;
    public int ManaCost => m_ManaCost;
    public int Attack => m_Attack;
    public int Health => m_Health;
    public IReadOnlyList<EffectDefinition> Effects => m_Effects;
    public IReadOnlyList<string> Tags => m_Tags;
}
```

### CardInstance (Runtime Copy)

Mutable runtime representation. Created from a definition when a card enters the game.

```csharp
public sealed class CardInstance
{
    private static int s_NextInstanceId;

    public int InstanceId { get; }
    public CardDefinition Definition { get; }
    public int CurrentAttack { get; set; }
    public int CurrentHealth { get; set; }
    public int CurrentManaCost { get; set; }
    public bool IsSilenced { get; set; }
    public List<StatModifier> Modifiers { get; } = new();

    public CardInstance(CardDefinition definition)
    {
        InstanceId = s_NextInstanceId++;
        Definition = definition;
        CurrentAttack = definition.Attack;
        CurrentHealth = definition.Health;
        CurrentManaCost = definition.ManaCost;
    }

    public bool IsAlive => CurrentHealth > 0;
}
```

### Card Factory

```csharp
public sealed class CardFactory
{
    private readonly Dictionary<string, CardDefinition> m_Registry = new();

    public void RegisterAll(IReadOnlyList<CardDefinition> definitions)
    {
        for (int cardIndex = 0; cardIndex < definitions.Count; cardIndex++)
        {
            m_Registry[definitions[cardIndex].CardId] = definitions[cardIndex];
        }
    }

    public CardInstance CreateInstance(string cardId)
    {
        if (!m_Registry.TryGetValue(cardId, out CardDefinition definition))
        {
            throw new System.ArgumentException($"Unknown card ID: {cardId}");
        }
        return new CardInstance(definition);
    }
}
```

---

## Zone System

Cards exist in exactly one zone at a time. Moving a card between zones is the fundamental operation.

### Zone Model

```csharp
public enum CardZone { Deck, Hand, Battlefield, Graveyard, Exile }

public sealed class ZoneModel
{
    private readonly Dictionary<CardZone, List<CardInstance>> m_Zones = new()
    {
        { CardZone.Deck, new List<CardInstance>() },
        { CardZone.Hand, new List<CardInstance>() },
        { CardZone.Battlefield, new List<CardInstance>() },
        { CardZone.Graveyard, new List<CardInstance>() },
        { CardZone.Exile, new List<CardInstance>() },
    };

    private readonly Dictionary<CardZone, int> m_Capacities = new()
    {
        { CardZone.Hand, 10 },
        { CardZone.Battlefield, 7 },
    };

    public IReadOnlyList<CardInstance> GetZone(CardZone zone) => m_Zones[zone];

    public int GetCount(CardZone zone) => m_Zones[zone].Count;

    public bool HasCapacity(CardZone zone)
    {
        if (!m_Capacities.TryGetValue(zone, out int cap)) return true;
        return m_Zones[zone].Count < cap;
    }

    public void AddToZone(CardZone zone, CardInstance card) => m_Zones[zone].Add(card);
    public void RemoveFromZone(CardZone zone, CardInstance card) => m_Zones[zone].Remove(card);
    public void InsertAtTop(CardZone zone, CardInstance card) => m_Zones[zone].Insert(0, card);
}
```

### Zone Transfer System

```csharp
// Message fired whenever a card changes zones
public readonly struct CardZoneChangedMessage
{
    public readonly CardInstance Card;
    public readonly CardZone FromZone;
    public readonly CardZone ToZone;

    public CardZoneChangedMessage(CardInstance card, CardZone fromZone, CardZone toZone)
    {
        Card = card;
        FromZone = fromZone;
        ToZone = toZone;
    }
}

public sealed class ZoneSystem : IDisposable
{
    private readonly ZoneModel m_Model;
    private readonly IPublisher<CardZoneChangedMessage> m_ZonePublisher;

    [Inject]
    public ZoneSystem(ZoneModel model, IPublisher<CardZoneChangedMessage> zonePublisher)
    {
        m_Model = model;
        m_ZonePublisher = zonePublisher;
    }

    public bool TryMoveCard(CardInstance card, CardZone from, CardZone to)
    {
        if (!m_Model.HasCapacity(to)) return false;

        m_Model.RemoveFromZone(from, card);
        m_Model.AddToZone(to, card);
        m_ZonePublisher.Publish(new CardZoneChangedMessage(card, from, to));
        return true;
    }

    public void Dispose() { }
}
```

Zone capacity matters: when the hand is full, drawn cards go directly to the graveyard ("overdraw" or "burn"). The battlefield cap prevents unlimited board flooding.

---

## Turn State Machine

### Turn Phases

```csharp
public enum TurnPhase { DrawPhase, MainPhase, CombatPhase, EndPhase }

public readonly struct TurnPhaseChangedMessage
{
    public readonly TurnPhase Phase;
    public readonly int PlayerId;

    public TurnPhaseChangedMessage(TurnPhase phase, int playerId)
    {
        Phase = phase;
        PlayerId = playerId;
    }
}

public sealed class TurnSystem : IDisposable
{
    private readonly IPublisher<TurnPhaseChangedMessage> m_PhasePublisher;
    private int m_CurrentPlayerId;
    private int m_TurnNumber;
    private TurnPhase m_CurrentPhase;

    private const int k_CardsPerDraw = 1;
    private const int k_MaxMana = 10;

    [Inject]
    public TurnSystem(IPublisher<TurnPhaseChangedMessage> phasePublisher)
    {
        m_PhasePublisher = phasePublisher;
    }

    public int CurrentPlayerId => m_CurrentPlayerId;
    public int TurnNumber => m_TurnNumber;
    public TurnPhase CurrentPhase => m_CurrentPhase;

    public void StartTurn(int playerId)
    {
        m_CurrentPlayerId = playerId;
        m_TurnNumber++;
        SetPhase(TurnPhase.DrawPhase);
    }

    public void AdvancePhase()
    {
        TurnPhase next = m_CurrentPhase switch
        {
            TurnPhase.DrawPhase => TurnPhase.MainPhase,
            TurnPhase.MainPhase => TurnPhase.CombatPhase,
            TurnPhase.CombatPhase => TurnPhase.EndPhase,
            _ => TurnPhase.DrawPhase,
        };
        SetPhase(next);
    }

    private void SetPhase(TurnPhase phase)
    {
        m_CurrentPhase = phase;
        m_PhasePublisher.Publish(new TurnPhaseChangedMessage(phase, m_CurrentPlayerId));
    }

    public void Dispose() { }
}
```

### Mana / Resource System

```csharp
public sealed class ManaModel
{
    public int MaxMana { get; set; }
    public int CurrentMana { get; set; }

    public bool CanSpend(int amount) => CurrentMana >= amount;

    public void Spend(int amount) => CurrentMana -= amount;

    public void RefreshForTurn(int turnNumber)
    {
        MaxMana = Mathf.Min(turnNumber, 10);
        CurrentMana = MaxMana;
    }
}
```

Resources refresh each turn. Many card games grant +1 max mana per turn up to a cap. The `RefreshForTurn` call happens during the DrawPhase before the player acts.

---

## Effect Resolution

### Effect Definition

```csharp
public enum EffectType { Damage, Heal, DrawCards, BuffStats, Summon, DestroyMinion, Silence, GainMana }
public enum TargetMode { SingleEnemy, SingleFriendly, AllEnemies, AllFriendlies, AllMinions, Self, Random, Hero }
public enum TriggerTiming { OnPlay, OnDeath, EndOfTurn, StartOfTurn, OnDamaged, OnHeal }

[System.Serializable]
public sealed class EffectDefinition
{
    [SerializeField] private EffectType m_Type;
    [SerializeField] private TargetMode m_TargetMode;
    [SerializeField] private TriggerTiming m_Timing;
    [SerializeField] private int m_Value;
    [SerializeField] private int m_Duration;

    public EffectType Type => m_Type;
    public TargetMode TargetMode => m_TargetMode;
    public TriggerTiming Timing => m_Timing;
    public int Value => m_Value;
    public int Duration => m_Duration;
}
```

### Effect Stack (LIFO Resolution)

Effects resolve in a last-in-first-out stack. When a card is played, its effects are pushed. If an effect triggers another effect (e.g., "on damage, draw a card"), the new effect is pushed on top and resolves first.

```csharp
public sealed class EffectResolver : IDisposable
{
    private readonly Stack<PendingEffect> m_EffectStack = new();
    private readonly ZoneSystem m_ZoneSystem;
    private readonly IPublisher<EffectResolvedMessage> m_ResolvedPublisher;

    [Inject]
    public EffectResolver(ZoneSystem zoneSystem, IPublisher<EffectResolvedMessage> resolvedPublisher)
    {
        m_ZoneSystem = zoneSystem;
        m_ResolvedPublisher = resolvedPublisher;
    }

    public void PushEffect(EffectDefinition effect, CardInstance source, List<CardInstance> targets)
    {
        m_EffectStack.Push(new PendingEffect(effect, source, targets));
    }

    public void ResolveAll()
    {
        while (m_EffectStack.Count > 0)
        {
            PendingEffect pending = m_EffectStack.Pop();
            ExecuteEffect(pending);
        }
    }

    private void ExecuteEffect(PendingEffect pending)
    {
        for (int targetIndex = 0; targetIndex < pending.Targets.Count; targetIndex++)
        {
            CardInstance target = pending.Targets[targetIndex];
            switch (pending.Effect.Type)
            {
                case EffectType.Damage:
                    target.CurrentHealth -= pending.Effect.Value;
                    if (!target.IsAlive)
                    {
                        m_ZoneSystem.TryMoveCard(target, CardZone.Battlefield, CardZone.Graveyard);
                    }
                    break;
                case EffectType.Heal:
                    target.CurrentHealth = Mathf.Min(
                        target.CurrentHealth + pending.Effect.Value,
                        target.Definition.Health);
                    break;
                case EffectType.BuffStats:
                    target.CurrentAttack += pending.Effect.Value;
                    target.CurrentHealth += pending.Effect.Value;
                    break;
                case EffectType.Silence:
                    target.IsSilenced = true;
                    target.Modifiers.Clear();
                    break;
            }
        }
        m_ResolvedPublisher.Publish(new EffectResolvedMessage(pending.Effect, pending.Source));
    }

    public void Dispose() { }
}

public readonly struct PendingEffect
{
    public readonly EffectDefinition Effect;
    public readonly CardInstance Source;
    public readonly List<CardInstance> Targets;

    public PendingEffect(EffectDefinition effect, CardInstance source, List<CardInstance> targets)
    {
        Effect = effect;
        Source = source;
        Targets = targets;
    }
}

public readonly struct EffectResolvedMessage
{
    public readonly EffectDefinition Effect;
    public readonly CardInstance Source;

    public EffectResolvedMessage(EffectDefinition effect, CardInstance source)
    {
        Effect = effect;
        Source = source;
    }
}
```

### Targeting System

Targeting depends on `TargetMode`. For `SingleEnemy` or `SingleFriendly`, the UI must let the player pick a target before the effect resolves. For `AllEnemies`, the system auto-selects all minions in the opponent's battlefield zone. For `Random`, use a seeded random to pick from valid targets.

```csharp
public sealed class TargetingSystem
{
    private readonly System.Random m_Random;

    public TargetingSystem(int seed)
    {
        m_Random = new System.Random(seed);
    }

    public List<CardInstance> ResolveTargets(
        TargetMode mode,
        ZoneModel allyZones,
        ZoneModel enemyZones,
        CardInstance selectedTarget = null)
    {
        List<CardInstance> targets = new();
        switch (mode)
        {
            case TargetMode.SingleEnemy:
                if (selectedTarget != null) targets.Add(selectedTarget);
                break;
            case TargetMode.AllEnemies:
                targets.AddRange(enemyZones.GetZone(CardZone.Battlefield));
                break;
            case TargetMode.AllFriendlies:
                targets.AddRange(allyZones.GetZone(CardZone.Battlefield));
                break;
            case TargetMode.Random:
                IReadOnlyList<CardInstance> pool = enemyZones.GetZone(CardZone.Battlefield);
                if (pool.Count > 0)
                {
                    targets.Add(pool[m_Random.Next(pool.Count)]);
                }
                break;
        }
        return targets;
    }
}
```

---

## Hand and Deck Management

### Fisher-Yates Shuffle

Always use Fisher-Yates for unbiased shuffling. Seed the random for deterministic replay.

```csharp
public sealed class DeckSystem : IDisposable
{
    private readonly ZoneModel m_ZoneModel;
    private readonly ZoneSystem m_ZoneSystem;
    private readonly System.Random m_Random;

    [Inject]
    public DeckSystem(ZoneModel zoneModel, ZoneSystem zoneSystem)
    {
        m_ZoneModel = zoneModel;
        m_ZoneSystem = zoneSystem;
        m_Random = new System.Random();
    }

    public void Shuffle()
    {
        IReadOnlyList<CardInstance> deck = m_ZoneModel.GetZone(CardZone.Deck);
        // Copy to mutable list for shuffle
        List<CardInstance> mutable = new(deck);
        for (int cardIndex = mutable.Count - 1; cardIndex > 0; cardIndex--)
        {
            int swapIndex = m_Random.Next(cardIndex + 1);
            (mutable[cardIndex], mutable[swapIndex]) = (mutable[swapIndex], mutable[cardIndex]);
        }
        // Rebuild zone
        for (int cardIndex = mutable.Count - 1; cardIndex >= 0; cardIndex--)
        {
            m_ZoneModel.RemoveFromZone(CardZone.Deck, mutable[cardIndex]);
        }
        for (int cardIndex = 0; cardIndex < mutable.Count; cardIndex++)
        {
            m_ZoneModel.AddToZone(CardZone.Deck, mutable[cardIndex]);
        }
    }

    public bool TryDraw(int count = 1)
    {
        for (int drawIndex = 0; drawIndex < count; drawIndex++)
        {
            IReadOnlyList<CardInstance> deck = m_ZoneModel.GetZone(CardZone.Deck);
            if (deck.Count == 0) return false;

            CardInstance top = deck[0];
            if (!m_ZoneModel.HasCapacity(CardZone.Hand))
            {
                // Overdraw: card is burned
                m_ZoneSystem.TryMoveCard(top, CardZone.Deck, CardZone.Graveyard);
            }
            else
            {
                m_ZoneSystem.TryMoveCard(top, CardZone.Deck, CardZone.Hand);
            }
        }
        return true;
    }

    public void Dispose() { }
}
```

### Hand Size Limit

When the hand is at capacity, drawn cards are burned (moved directly to graveyard). Some games let the player choose which card to discard. Implement discard-to-hand-size as an end-of-turn check:

```csharp
public void EnforceHandLimit(int maxHandSize)
{
    IReadOnlyList<CardInstance> hand = m_ZoneModel.GetZone(CardZone.Hand);
    while (hand.Count > maxHandSize)
    {
        // Discard the rightmost card (or prompt player to choose)
        CardInstance discard = hand[hand.Count - 1];
        m_ZoneSystem.TryMoveCard(discard, CardZone.Hand, CardZone.Graveyard);
    }
}
```

### Mulligan System

At game start, show the player their opening hand. They select cards to replace, those go back into the deck, the deck shuffles, and replacement cards are drawn.

```csharp
public void PerformMulligan(List<CardInstance> cardsToReplace)
{
    for (int cardIndex = 0; cardIndex < cardsToReplace.Count; cardIndex++)
    {
        m_ZoneSystem.TryMoveCard(cardsToReplace[cardIndex], CardZone.Hand, CardZone.Deck);
    }
    Shuffle();
    TryDraw(cardsToReplace.Count);
}
```

---

## Card UI and Animation

### Hand Layout (Arc Formation)

Cards in hand should fan out in an arc. Calculate each card's position and rotation based on its index within the hand.

```csharp
public sealed class HandLayoutView : MonoBehaviour
{
    [SerializeField] private RectTransform m_HandContainer;
    [SerializeField] private float m_ArcRadius = 1200f;
    [SerializeField] private float m_ArcAngle = 30f;
    [SerializeField] private float m_CardSpacing = 120f;

    private readonly List<RectTransform> m_CardSlots = new();

    public void LayoutCards(int cardCount)
    {
        float totalWidth = (cardCount - 1) * m_CardSpacing;
        float startX = -totalWidth * 0.5f;

        for (int cardIndex = 0; cardIndex < cardCount; cardIndex++)
        {
            if (cardIndex >= m_CardSlots.Count) break;

            float normalizedPos = cardCount > 1
                ? (float)cardIndex / (cardCount - 1)
                : 0.5f;
            float angle = Mathf.Lerp(m_ArcAngle, -m_ArcAngle, normalizedPos);
            float x = startX + cardIndex * m_CardSpacing;
            float y = Mathf.Cos(normalizedPos * Mathf.PI) * m_ArcRadius * 0.05f;

            RectTransform slot = m_CardSlots[cardIndex];
            slot.anchoredPosition = new Vector2(x, y);
            slot.localRotation = Quaternion.Euler(0f, 0f, angle);
        }
    }
}
```

### Card Hover and Zoom

When the player hovers over a card in hand, scale it up and raise it above its neighbors. Use DOTween or manual lerp.

```csharp
public sealed class CardHoverView : MonoBehaviour
{
    [SerializeField] private RectTransform m_CardRect;
    [SerializeField] private float m_HoverScale = 1.4f;
    [SerializeField] private float m_HoverYOffset = 80f;
    [SerializeField] private float m_TransitionSpeed = 10f;

    private Vector2 m_OriginalPosition;
    private Vector3 m_OriginalScale;
    private bool m_IsHovered;

    private void Awake()
    {
        m_OriginalScale = m_CardRect.localScale;
    }

    public void SetHovered(bool hovered)
    {
        m_IsHovered = hovered;
        if (hovered)
        {
            m_OriginalPosition = m_CardRect.anchoredPosition;
        }
    }

    private void Update()
    {
        Vector3 targetScale = m_IsHovered
            ? m_OriginalScale * m_HoverScale
            : m_OriginalScale;
        Vector2 targetPos = m_IsHovered
            ? m_OriginalPosition + new Vector2(0f, m_HoverYOffset)
            : m_OriginalPosition;

        m_CardRect.localScale = Vector3.Lerp(
            m_CardRect.localScale, targetScale, Time.deltaTime * m_TransitionSpeed);
        m_CardRect.anchoredPosition = Vector2.Lerp(
            m_CardRect.anchoredPosition, targetPos, Time.deltaTime * m_TransitionSpeed);
    }
}
```

### Drag to Play

Use Unity's EventSystem drag interfaces. When the player drags a card from hand onto the battlefield zone, attempt to play it.

### Play-to-Field Transition

Animate the card from its hand position to its target slot on the battlefield. During animation, the card is in a transitional state -- it has already been removed from the hand Model but the View is still animating. Use async UniTask for the animation wait:

```csharp
public async UniTask AnimateCardPlay(RectTransform card, Vector2 targetPosition, CancellationToken token)
{
    float elapsed = 0f;
    float duration = 0.3f;
    Vector2 start = card.anchoredPosition;

    while (elapsed < duration)
    {
        elapsed += Time.deltaTime;
        float t = elapsed / duration;
        card.anchoredPosition = Vector2.Lerp(start, targetPosition, t);
        await UniTask.Yield(token);
    }
    card.anchoredPosition = targetPosition;
}
```

---

## Deck Building

### Deck Definition

```csharp
[CreateAssetMenu(menuName = "CardGame/Deck Definition")]
public sealed class DeckDefinition : ScriptableObject
{
    [SerializeField] private string m_DeckName;
    [SerializeField] private List<DeckEntry> m_Cards;

    public string DeckName => m_DeckName;
    public IReadOnlyList<DeckEntry> Cards => m_Cards;

    public int TotalCardCount
    {
        get
        {
            int total = 0;
            for (int entryIndex = 0; entryIndex < m_Cards.Count; entryIndex++)
            {
                total += m_Cards[entryIndex].Count;
            }
            return total;
        }
    }
}

[System.Serializable]
public sealed class DeckEntry
{
    [SerializeField] private CardDefinition m_Card;
    [SerializeField] private int m_Count;

    public CardDefinition Card => m_Card;
    public int Count => m_Count;
}
```

### Deck Validation

```csharp
public sealed class DeckValidator
{
    private const int k_MinDeckSize = 30;
    private const int k_MaxDeckSize = 40;
    private const int k_MaxCopiesCommon = 3;
    private const int k_MaxCopiesRare = 2;
    private const int k_MaxCopiesLegendary = 1;

    public bool Validate(DeckDefinition deck, out string error)
    {
        int total = deck.TotalCardCount;
        if (total < k_MinDeckSize)
        {
            error = $"Deck has {total} cards, minimum is {k_MinDeckSize}";
            return false;
        }
        if (total > k_MaxDeckSize)
        {
            error = $"Deck has {total} cards, maximum is {k_MaxDeckSize}";
            return false;
        }

        for (int entryIndex = 0; entryIndex < deck.Cards.Count; entryIndex++)
        {
            DeckEntry entry = deck.Cards[entryIndex];
            int maxCopies = entry.Card.Rarity switch
            {
                CardRarity.Legendary => k_MaxCopiesLegendary,
                CardRarity.Rare => k_MaxCopiesRare,
                _ => k_MaxCopiesCommon,
            };

            if (entry.Count > maxCopies)
            {
                error = $"{entry.Card.DisplayName} ({entry.Card.Rarity}): {entry.Count} copies exceeds max {maxCopies}";
                return false;
            }
        }

        error = null;
        return true;
    }
}
```

### Collection Tracking

Track which cards the player owns and how many copies. The collection is separate from any deck -- decks reference cards from the collection.

```csharp
public sealed class CollectionModel
{
    private readonly Dictionary<string, int> m_OwnedCards = new();

    public int GetOwnedCount(string cardId) =>
        m_OwnedCards.TryGetValue(cardId, out int count) ? count : 0;

    public void AddCard(string cardId, int amount = 1)
    {
        m_OwnedCards.TryGetValue(cardId, out int current);
        m_OwnedCards[cardId] = current + amount;
    }

    public bool RemoveCard(string cardId, int amount = 1)
    {
        if (!m_OwnedCards.TryGetValue(cardId, out int current) || current < amount)
            return false;
        m_OwnedCards[cardId] = current - amount;
        return true;
    }
}
```

---

## Common Pitfalls

### Effect Resolution Order

Effects that trigger other effects can cause infinite loops. Always add a recursion depth limit or a "processing" flag to prevent re-entrant triggers. Test chains like: "on death, deal 1 damage to all" killing another minion that also has an on-death trigger.

### Card Reference vs Copy

Never pass `CardDefinition` references as runtime state. Always create a `CardInstance` from the definition. Two cards with the same definition must have independent mutable state (health, buffs). Confusing the two leads to one buff affecting all copies of a card.

### UI Desync with Model

The zone Model is the source of truth. When an animation is playing (e.g., card flying to battlefield), the Model has already updated. The View catches up visually. Never let the View block or delay Model updates -- always update Model first, then animate.

### Shuffle Predictability

Use a seeded `System.Random` rather than `UnityEngine.Random.Range`. Seeded random enables deterministic replays and server-authoritative games. Share the seed between client and server for validation.

### Turn Timer Exploits

If implementing a turn timer, enforce it server-side. Client-side timers can be manipulated. When the timer expires, auto-pass the turn even if the player has pending actions.

---

## Performance

### Card Instance Pooling

Pool `CardInstance` objects and their associated View GameObjects. A match rarely has more than 50-60 card instances alive simultaneously. Pre-allocate a pool of that size.

### Partial Hand Re-render

When a single card is played from hand, do not rebuild the entire hand layout. Remove the played card's slot and re-position only the remaining cards. This avoids unnecessary RectTransform churn.

### Batch Effect Resolution

When multiple effects trigger simultaneously (e.g., "deal 1 damage to all enemies"), resolve all damage in a single pass before checking for deaths. This prevents the death-check from running N times. Collect all dead minions, then move them to graveyard in one batch.

### Avoid String Lookups

Use integer IDs or enum values for card identification in hot paths. String comparisons in effect resolution loops cause unnecessary allocations and cache misses. Reserve string IDs for serialization and display.

### Message Throttling

Zone change messages can fire rapidly during draw or AoE resolution. If the View subscribes to every individual zone change, it may rebuild the UI many times per frame. Batch zone changes within a single resolution pass and publish a summary message at the end, or use a dirty flag on the View that triggers a single rebuild in LateUpdate.
