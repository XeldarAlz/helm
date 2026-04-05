---
name: ui-toolkit
description: "UI Toolkit — UXML document structure, USS styling (CSS-like), UQuery, data binding, ListView virtualization, custom visual elements."
globs: ["**/*.uxml", "**/*.uss", "**/UIDocument*"]
---

# UI Toolkit

## UXML Structure

```xml
<ui:UXML xmlns:ui="UnityEngine.UIElements">
    <ui:Style src="styles.uss" />

    <ui:VisualElement class="screen">
        <ui:Label text="Game Title" class="title" />

        <ui:VisualElement class="button-container">
            <ui:Button text="Play" name="btn-play" class="menu-btn" />
            <ui:Button text="Settings" name="btn-settings" class="menu-btn" />
            <ui:Button text="Quit" name="btn-quit" class="menu-btn danger" />
        </ui:VisualElement>

        <ui:Slider label="Volume" name="volume-slider" low-value="0" high-value="1" value="0.8" />
        <ui:Toggle label="Fullscreen" name="fullscreen-toggle" />
    </ui:VisualElement>
</ui:UXML>
```

## USS Styling (CSS-like)

```css
.screen {
    flex-grow: 1;
    align-items: center;
    justify-content: center;
    background-color: rgba(0, 0, 0, 0.8);
}

.title {
    font-size: 48px;
    color: white;
    -unity-font-style: bold;
    margin-bottom: 40px;
}

.menu-btn {
    width: 250px;
    height: 50px;
    margin: 8px;
    font-size: 20px;
    background-color: rgb(60, 60, 60);
    color: white;
    border-radius: 8px;
    border-width: 0;
    transition-duration: 0.2s;
}

.menu-btn:hover {
    background-color: rgb(80, 80, 80);
    scale: 1.05;
}

.menu-btn:active {
    background-color: rgb(40, 40, 40);
}

.danger {
    color: rgb(255, 100, 100);
}
```

### Key USS Differences from CSS
- Flex layout only (no floats, no grid)
- Use `-unity-` prefix for Unity-specific properties
- Colors: `rgb()`, `rgba()`, `#hex`
- Transitions: `transition-duration`, `transition-property`
- No `em`/`rem` — use `px` or `%`

## Controller Script

```csharp
public sealed class MainMenuController : MonoBehaviour
{
    [SerializeField] private UIDocument m_Document;

    private void OnEnable()
    {
        VisualElement root = m_Document.rootVisualElement;

        root.Q<Button>("btn-play").clicked += OnPlayClicked;
        root.Q<Button>("btn-settings").clicked += OnSettingsClicked;
        root.Q<Button>("btn-quit").clicked += OnQuitClicked;

        Slider volumeSlider = root.Q<Slider>("volume-slider");
        volumeSlider.RegisterValueChangedCallback(evt => OnVolumeChanged(evt.newValue));
    }

    private void OnPlayClicked() { /* Load game scene */ }
    private void OnSettingsClicked() { /* Show settings panel */ }
    private void OnQuitClicked() { Application.Quit(); }
    private void OnVolumeChanged(float value) { /* Set audio volume */ }
}
```

## UQuery

```csharp
VisualElement root = m_Document.rootVisualElement;

// By name
Button playBtn = root.Q<Button>("btn-play");

// By class
var allButtons = root.Query<Button>(className: "menu-btn").ToList();

// By type
var allLabels = root.Query<Label>().ToList();

// Nested query
var containerBtns = root.Q("button-container").Query<Button>().ToList();
```

## ListView (Virtualized Scrolling)

```csharp
ListView listView = root.Q<ListView>("inventory-list");
listView.makeItem = () => new Label(); // Create UI element
listView.bindItem = (element, index) =>
{
    ((Label)element).text = m_Items[index].Name;
};
listView.itemsSource = m_Items;
listView.fixedItemHeight = 40;
listView.selectionType = SelectionType.Single;
listView.selectionChanged += OnSelectionChanged;
```

## Custom Visual Element

```csharp
public sealed class HealthBar : VisualElement
{
    private VisualElement m_Fill;

    public float Value
    {
        set => m_Fill.style.width = new Length(value * 100f, LengthUnit.Percent);
    }

    public HealthBar()
    {
        AddToClassList("health-bar");
        m_Fill = new VisualElement();
        m_Fill.AddToClassList("health-fill");
        Add(m_Fill);
    }

    // Required for UXML instantiation
    public new sealed class UxmlFactory : UxmlFactory<HealthBar> { }
}
```

## Event System

```csharp
// Register callbacks
element.RegisterCallback<ClickEvent>(evt => { });
element.RegisterCallback<PointerEnterEvent>(evt => { });
element.RegisterCallback<KeyDownEvent>(evt => { });

// Unregister
element.UnregisterCallback<ClickEvent>(handler);
```

## Advanced Layout

### Flex Grow, Shrink, and Basis

Flex properties control how elements share available space within a container:

```css
/* flex-grow: how much extra space this element takes (relative to siblings) */
.sidebar { flex-grow: 0; width: 200px; }   /* fixed width, no growth */
.content { flex-grow: 1; }                  /* takes all remaining space */
.inspector { flex-grow: 0; width: 300px; }  /* fixed width, no growth */

/* flex-shrink: how much this element shrinks when space is tight */
.important { flex-shrink: 0; }  /* never shrink below natural size */
.optional { flex-shrink: 1; }   /* shrink proportionally if needed */

/* flex-basis: starting size before grow/shrink is applied */
.panel { flex-basis: 25%; }     /* start at 25% of parent, then grow/shrink */
```

A common pattern for equal-width columns:
```css
.column { flex-grow: 1; flex-basis: 0; }
```

### Absolute Positioning for Overlays

Use `position: absolute` for elements that overlay the flex layout (tooltips, popups,
floating damage numbers):

```css
.tooltip {
    position: absolute;
    left: 50%;
    top: -40px;
    background-color: rgba(0, 0, 0, 0.9);
    color: white;
    padding: 8px 12px;
    border-radius: 4px;
}

.notification-badge {
    position: absolute;
    right: -8px;
    top: -8px;
    width: 20px;
    height: 20px;
    border-radius: 10px;
    background-color: red;
}
```

Absolute elements are positioned relative to their nearest positioned ancestor.

### Min/Max Width/Height Constraints

Constrain element sizing for responsive behavior:

```css
.dialog {
    min-width: 300px;
    max-width: 80%;
    min-height: 200px;
    max-height: 90%;
    flex-grow: 1;
}

.inventory-slot {
    min-width: 48px;
    min-height: 48px;
    max-width: 64px;
    max-height: 64px;
}
```

### Percentage-Based Responsive Sizing

Use percentages for layouts that adapt to screen size:

```css
.hud-bar {
    width: 30%;
    height: 4%;
    margin-left: 2%;
    margin-top: 2%;
}

.modal-overlay {
    width: 100%;
    height: 100%;
    position: absolute;
    background-color: rgba(0, 0, 0, 0.5);
    align-items: center;
    justify-content: center;
}

.modal-content {
    width: 60%;
    height: 70%;
    background-color: rgb(30, 30, 30);
    border-radius: 12px;
    padding: 20px;
}
```

## Theming System

### USS Custom Properties (Variables)

Define reusable design tokens as USS variables:

```css
:root {
    --color-primary: rgb(66, 133, 244);
    --color-primary-hover: rgb(100, 160, 255);
    --color-surface: rgb(30, 30, 30);
    --color-surface-light: rgb(50, 50, 50);
    --color-text: rgb(230, 230, 230);
    --color-text-muted: rgb(150, 150, 150);
    --color-danger: rgb(234, 67, 53);
    --spacing-sm: 4px;
    --spacing-md: 8px;
    --spacing-lg: 16px;
    --radius-sm: 4px;
    --radius-md: 8px;
    --font-size-body: 16px;
    --font-size-heading: 24px;
}

.btn {
    background-color: var(--color-primary);
    color: var(--color-text);
    padding: var(--spacing-md) var(--spacing-lg);
    border-radius: var(--radius-md);
    font-size: var(--font-size-body);
}

.btn:hover {
    background-color: var(--color-primary-hover);
}
```

### Runtime Theme Switching

Load different USS files at runtime to change the entire UI appearance:

```csharp
public sealed class ThemeManager : MonoBehaviour
{
    [SerializeField] private UIDocument m_Document;
    [SerializeField] private StyleSheet m_DarkTheme;
    [SerializeField] private StyleSheet m_LightTheme;

    private StyleSheet m_ActiveTheme;

    public void SetDarkMode()
    {
        SwapTheme(m_DarkTheme);
    }

    public void SetLightMode()
    {
        SwapTheme(m_LightTheme);
    }

    private void SwapTheme(StyleSheet newTheme)
    {
        VisualElement root = m_Document.rootVisualElement;
        if (m_ActiveTheme != null)
        {
            root.styleSheets.Remove(m_ActiveTheme);
        }
        root.styleSheets.Add(newTheme);
        m_ActiveTheme = newTheme;
    }
}
```

### Dark / Light Mode Pattern

Create two USS files that redefine the same variables:

```css
/* dark-theme.uss */
:root {
    --color-background: rgb(18, 18, 18);
    --color-surface: rgb(30, 30, 30);
    --color-text: rgb(230, 230, 230);
    --color-border: rgb(60, 60, 60);
}

/* light-theme.uss */
:root {
    --color-background: rgb(245, 245, 245);
    --color-surface: rgb(255, 255, 255);
    --color-text: rgb(30, 30, 30);
    --color-border: rgb(200, 200, 200);
}
```

All UI elements referencing `var(--color-background)` update automatically when
the stylesheet is swapped.

### Theme Asset Loading

For games with many themes (player-selectable UI skins), load theme assets
from Addressables or Resources:

```csharp
public sealed class ThemeLoader
{
    private readonly UIDocument m_Document;
    private StyleSheet m_CurrentTheme;

    [Inject]
    public ThemeLoader(UIDocument document)
    {
        m_Document = document;
    }

    public async UniTask LoadThemeAsync(string themeAddress, CancellationToken token)
    {
        StyleSheet newTheme = await Addressables.LoadAssetAsync<StyleSheet>(themeAddress)
            .ToUniTask(cancellationToken: token);

        VisualElement root = m_Document.rootVisualElement;
        if (m_CurrentTheme != null)
        {
            root.styleSheets.Remove(m_CurrentTheme);
        }
        root.styleSheets.Add(newTheme);
        m_CurrentTheme = newTheme;
    }
}
```

## Performance with Large Lists

### ListView Virtualization Tuning

ListView only creates enough visual elements to fill the visible area plus a small
overflow buffer. Key settings:

```csharp
ListView listView = root.Q<ListView>("item-list");
listView.fixedItemHeight = 40;            // MUST set for virtualization to work
listView.virtualizationMethod = CollectionVirtualizationMethod.FixedHeight;
listView.showAlternatingRowBackgrounds = AlternatingRowBackground.ContentOnly;
```

Use `fixedItemHeight` when all items are the same height (most performant).
Use `DynamicHeight` only when items genuinely vary in size.

### makeItem / bindItem Optimization

The `makeItem` callback creates reusable visual element templates. The `bindItem`
callback populates them with data. Avoid allocations in both:

```csharp
// GOOD — makeItem creates the template once, bindItem only sets values
listView.makeItem = () =>
{
    var row = new VisualElement();
    row.AddToClassList("list-row");

    var icon = new VisualElement();
    icon.AddToClassList("list-icon");
    icon.name = "icon";
    row.Add(icon);

    var label = new Label();
    label.name = "label";
    row.Add(label);

    return row;
};

listView.bindItem = (element, index) =>
{
    ItemData item = m_Items[index];
    element.Q<Label>("label").text = item.DisplayName;
    element.Q("icon").style.backgroundImage = new StyleBackground(item.Icon);
};

// BAD — allocates new elements in bindItem
listView.bindItem = (element, index) =>
{
    element.Clear();                          // destroys cached children
    element.Add(new Label(m_Items[index].Name)); // allocates every bind
};
```

### Batch Updates

When the data source changes, avoid rebuilding per item. Use `RefreshItems`
to trigger a single batch rebind:

```csharp
// After adding/removing items from the source list
m_Items.Add(newItem);
listView.RefreshItems(); // rebinds only visible items

// For full data source replacement
listView.itemsSource = newDataList;
listView.Rebuild(); // recreates the visual tree
```

### ScrollView vs ListView Decision

| Use ScrollView | Use ListView |
|----------------|-------------|
| < 50 static items | 50+ items or dynamic data |
| Complex mixed layouts | Uniform repeating rows |
| Nested scrollable areas | Inventory, leaderboard, chat log |
| No virtualization needed | Virtualization required for perf |

ScrollView creates all child elements immediately. ListView virtualizes and
recycles elements. For lists exceeding ~50 items, always use ListView.

## Input and Focus Management

### Focus Ring and Tab Order

UI Toolkit supports keyboard navigation via the focus ring. Set `tabIndex` to
control tab order:

```csharp
VisualElement root = m_Document.rootVisualElement;
root.Q<Button>("btn-play").tabIndex = 0;
root.Q<Button>("btn-settings").tabIndex = 1;
root.Q<Button>("btn-quit").tabIndex = 2;

// Exclude an element from tab navigation
root.Q("decorative-element").focusable = false;
```

Elements with `tabIndex >= 0` participate in tab navigation in ascending order.
Elements with `tabIndex = -1` are skipped by tab but can still receive programmatic focus.

### Keyboard Navigation

Handle arrow keys and Enter for gamepad/keyboard-friendly menus:

```csharp
root.RegisterCallback<NavigationMoveEvent>(evt =>
{
    // evt.direction is Up, Down, Left, Right
    // UI Toolkit handles focus movement automatically
    // Use this callback for custom behavior (e.g., grid navigation)
});

root.RegisterCallback<NavigationSubmitEvent>(evt =>
{
    // Enter/gamepad A pressed on focused element
    if (evt.target is Button button)
    {
        button.clickable.SimulateSingleClick(evt);
    }
});
```

### Custom Focusable Elements

Make a custom VisualElement focusable for keyboard navigation:

```csharp
public sealed class SelectableCard : VisualElement
{
    public SelectableCard()
    {
        focusable = true;
        AddToClassList("selectable-card");

        RegisterCallback<FocusInEvent>(evt => AddToClassList("focused"));
        RegisterCallback<FocusOutEvent>(evt => RemoveFromClassList("focused"));
        RegisterCallback<KeyDownEvent>(OnKeyDown);
    }

    private void OnKeyDown(KeyDownEvent evt)
    {
        if (evt.keyCode == KeyCode.Return || evt.keyCode == KeyCode.Space)
        {
            // Activate card
            evt.StopPropagation();
        }
    }

    public new sealed class UxmlFactory : UxmlFactory<SelectableCard> { }
}
```

### Input Capture (Prevent Event Propagation)

Stop events from bubbling up to parent elements:

```csharp
// Stop a click from reaching elements behind a popup
popupOverlay.RegisterCallback<ClickEvent>(evt =>
{
    evt.StopPropagation();
});

// Prevent scroll events from passing through a modal
modal.RegisterCallback<WheelEvent>(evt =>
{
    evt.StopPropagation();
});

// Use TrickleDown phase to intercept events before children see them
parent.RegisterCallback<PointerDownEvent>(evt =>
{
    // Handle before any child gets it
    evt.StopImmediatePropagation();
}, TrickleDownPhase.TrickleDown);
```

## Transitions and Animation

### USS Transition Property

Animate property changes with CSS-like transitions:

```css
.panel {
    opacity: 1;
    translate: 0 0;
    transition-property: opacity, translate;
    transition-duration: 0.3s, 0.3s;
    transition-timing-function: ease-in-out, ease-out;
}

.panel.hidden {
    opacity: 0;
    translate: 0 20px;
}

/* Shorthand form */
.fade-element {
    transition: opacity 0.2s ease, background-color 0.15s ease-in;
}
```

Supported animatable properties: `opacity`, `translate`, `scale`, `rotate`,
`background-color`, `color`, `border-color`, `width`, `height`, `margin-*`,
`padding-*`, `border-width`, `border-radius`, `flex-grow`, `flex-shrink`.

### Hover and Active State Animations

Combine pseudo-classes with transitions for interactive feedback:

```css
.inventory-slot {
    scale: 1;
    border-width: 2px;
    border-color: rgba(255, 255, 255, 0.1);
    transition: scale 0.15s ease-out, border-color 0.15s ease;
}

.inventory-slot:hover {
    scale: 1.08;
    border-color: rgba(255, 255, 255, 0.5);
}

.inventory-slot:active {
    scale: 0.95;
}

.inventory-slot.selected {
    border-color: var(--color-primary);
    border-width: 3px;
}
```

### Transform Origin for Scale Effects

Control the pivot point for scale and rotate transitions:

```css
/* Scale from center (default) */
.popup { transform-origin: center; }

/* Scale from top-left (dropdown menus) */
.dropdown { transform-origin: left top; }

/* Scale from bottom (toast notifications rising up) */
.toast { transform-origin: center bottom; }
```

### Class Toggle for State-Driven Animation

Toggle USS classes from C# to trigger transitions. This is the primary pattern
for UI Toolkit animation:

```csharp
public sealed class PanelAnimator : MonoBehaviour
{
    [SerializeField] private UIDocument m_Document;

    private VisualElement m_Panel;

    private void Awake()
    {
        m_Panel = m_Document.rootVisualElement.Q("panel");
    }

    public void Show()
    {
        m_Panel.RemoveFromClassList("hidden");
        m_Panel.AddToClassList("visible");
    }

    public void Hide()
    {
        m_Panel.RemoveFromClassList("visible");
        m_Panel.AddToClassList("hidden");
    }

    // Listen for transition end to clean up or chain animations
    public void SetupTransitionCallback()
    {
        m_Panel.RegisterCallback<TransitionEndEvent>(evt =>
        {
            if (m_Panel.ClassListContains("hidden"))
            {
                m_Panel.style.display = DisplayStyle.None;
            }
        });
    }
}
```

Corresponding USS:

```css
.panel {
    transition: opacity 0.3s ease, translate 0.3s ease-out;
}

.panel.visible {
    opacity: 1;
    translate: 0 0;
    display: flex;
}

.panel.hidden {
    opacity: 0;
    translate: 0 30px;
}
```

This pattern avoids runtime allocations and leverages the USS transition engine
for smooth, GPU-friendly animations.
