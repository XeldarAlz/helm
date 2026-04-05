---
name: animation
description: "Unity animation system — Animator controllers, layers, blend trees, state machine behaviors, root motion, animation events, Timeline."
globs: ["**/*.controller", "**/*Anim*.cs", "**/*.anim"]
---

# Animation System

## Animator Controller

### Parameters
```csharp
// Cache hash IDs — NEVER use string version in Update
private static readonly int k_SpeedHash = Animator.StringToHash("Speed");
private static readonly int k_JumpHash = Animator.StringToHash("Jump");
private static readonly int k_IsGroundedHash = Animator.StringToHash("IsGrounded");
private static readonly int k_AttackHash = Animator.StringToHash("Attack");

private void Update()
{
    m_Animator.SetFloat(k_SpeedHash, m_CurrentSpeed);
    m_Animator.SetBool(k_IsGroundedHash, m_IsGrounded);
}

// Triggers: fire once, auto-reset
public void Attack() => m_Animator.SetTrigger(k_AttackHash);
```

### Transition Settings
- **Has Exit Time:** animation finishes before transitioning (good for attacks, bad for instant response)
- **Fixed Duration:** transition time in seconds vs normalized
- **Transition Duration:** blend time (0 for instant, 0.1-0.25 for smooth)
- **Interruption Source:** which transitions can interrupt this one

### Layers
- Base Layer: locomotion (walk, run, idle)
- Upper Body Layer (Avatar Mask): aiming, attacks (overrides base)
- Additive Layer: breathing, hit reactions (adds on top)

## Blend Trees

**1D:** Speed parameter → walk/run blend
**2D Simple Directional:** X/Y input → directional movement (forward, back, strafe)
**2D Freeform:** more flexible placement of motion clips

## State Machine Behaviors

```csharp
public sealed class AttackStateBehavior : StateMachineBehaviour
{
    public override void OnStateEnter(Animator animator, AnimatorStateInfo stateInfo, int layerIndex)
    {
        // Enable hitbox
        animator.GetComponent<CombatSystem>().EnableHitbox();
    }

    public override void OnStateExit(Animator animator, AnimatorStateInfo stateInfo, int layerIndex)
    {
        // Disable hitbox
        animator.GetComponent<CombatSystem>().DisableHitbox();
    }
}
```

## Root Motion

- Enable `Apply Root Motion` on Animator
- Override in `OnAnimatorMove()` for custom control:

```csharp
private void OnAnimatorMove()
{
    // Use animation's root motion for position
    Vector3 deltaPosition = m_Animator.deltaPosition;
    transform.position += deltaPosition;

    // Use animation's rotation
    transform.rotation *= m_Animator.deltaRotation;
}
```

## Animation Events

Call methods from specific frames in animation clips:
```csharp
// Called from animation event on frame 12
public void OnFootstep()
{
    m_AudioSource.PlayOneShot(m_FootstepClip);
}

public void OnAttackHit()
{
    // Check hitbox collisions at this exact frame
}
```

## IK (Inverse Kinematics)

```csharp
private void OnAnimatorIK(int layerIndex)
{
    if (m_LookTarget != null)
    {
        m_Animator.SetLookAtWeight(1f, 0.3f, 0.6f, 1f);
        m_Animator.SetLookAtPosition(m_LookTarget.position);
    }

    // Foot IK for uneven terrain
    m_Animator.SetIKPositionWeight(AvatarIKGoal.LeftFoot, 1f);
    m_Animator.SetIKPosition(AvatarIKGoal.LeftFoot, m_LeftFootTarget);
}
```

## Timeline Integration

- Animation Track: play animation clips on any Animator
- Custom Playable: create custom Timeline clips with `PlayableAsset` + `PlayableBehaviour`
- Signal Track: fire events at specific times (similar to animation events but on Timeline)

## Avatar Masking

Avatar masks let you isolate body regions so different layers can control different parts independently.

### Upper/Lower Body Split

Create an AvatarMask asset (Assets > Create > Avatar Mask). In the Humanoid tab, toggle body parts:
- **Upper body mask:** enable Head, Left Arm, Right Arm, Left Hand, Right Hand. Disable everything else.
- **Lower body mask:** enable Left Leg, Right Leg, Root. Disable upper body parts.

Assign the mask to the Animator layer's Avatar Mask slot in the controller inspector.

### Layer Weight Blending (Shooting While Running)

```csharp
private static readonly int k_UpperBodyLayerIndex = 1;

private Animator m_Animator;

private void Awake()
{
    m_Animator = GetComponent<Animator>();
}

// Smoothly enable upper body override when aiming
public void SetAimWeight(float weight)
{
    m_Animator.SetLayerWeight(k_UpperBodyLayerIndex, weight);
}
```

Layer setup in the Animator Controller:
- Layer 0 (Base): full-body locomotion (walk, run, idle)
- Layer 1 (Upper Body Override): aim, shoot, reload — Avatar Mask = upper body, Blending = Override
- Layer 2 (Additive): hit flinch, breathing — Avatar Mask = none, Blending = Additive

### Additive Layer for Hit Reactions

Additive layers add motion on top of the base pose. Use them for:
- Hit flinch animations that overlay on any locomotion state
- Breathing that layers over idle/walk/run
- Weapon sway that adds subtle movement to aim poses

Set the layer to **Additive** blending and control intensity via `SetLayerWeight`. A weight of 0.5 plays the additive clip at half intensity.

## Advanced Blend Trees

### 2D Freeform Directional

Best for locomotion where speed and direction vary independently. Place clips at positions in 2D parameter space:

```
Parameter X: MoveX (strafe direction, -1 to 1)
Parameter Y: MoveY (forward/backward, -1 to 1)

Clip positions:
  Idle        → (0, 0)
  Walk Fwd    → (0, 0.5)
  Run Fwd     → (0, 1)
  Walk Back   → (0, -0.5)
  Strafe Left → (-1, 0)
  Strafe Right→ (1, 0)
```

Freeform Directional handles any direction. Freeform Cartesian is better when clips map to exact positions on a grid.

### Mirror Parameter

Enable **Mirror** on individual blend tree motions to reuse left-side animations for the right side. This halves the number of directional clips needed for strafing. Set the Mirror checkbox per motion in the blend tree inspector.

### Foot IK in Blend Trees

Enable **Foot IK** on each motion in the blend tree to keep feet planted correctly during blends between walk and run. Without Foot IK, feet can slide or float when two clips blend at different step cadences.

### Normalized Time Sync

When blending between walk and run clips with different lengths, enable **Adjust Time Scale** so both clips align their foot cycles. This prevents the walk foot planting at a different time than the run foot, eliminating the "moonwalk" effect during blend transitions.

## State Machine Behaviours (Advanced)

StateMachineBehaviour scripts attach to Animator states and receive lifecycle callbacks. Use them for tightly timed gameplay events.

### Complete Implementation Pattern

```csharp
public sealed class MeleeAttackBehaviour : StateMachineBehaviour
{
    [SerializeField] private float m_HitboxEnableTime = 0.2f;
    [SerializeField] private float m_HitboxDisableTime = 0.6f;

    private bool m_HitboxActive;

    public override void OnStateEnter(Animator animator, AnimatorStateInfo stateInfo, int layerIndex)
    {
        m_HitboxActive = false;
    }

    public override void OnStateUpdate(Animator animator, AnimatorStateInfo stateInfo, int layerIndex)
    {
        float normalizedTime = stateInfo.normalizedTime;

        if (!m_HitboxActive && normalizedTime >= m_HitboxEnableTime)
        {
            m_HitboxActive = true;
            // Cache this reference — GetComponent in OnStateEnter, store in field
            animator.GetComponent<HitboxController>().EnableHitbox();
        }

        if (m_HitboxActive && normalizedTime >= m_HitboxDisableTime)
        {
            m_HitboxActive = false;
            animator.GetComponent<HitboxController>().DisableHitbox();
        }
    }

    public override void OnStateExit(Animator animator, AnimatorStateInfo stateInfo, int layerIndex)
    {
        // Safety: always disable on exit in case of interruption
        if (m_HitboxActive)
        {
            m_HitboxActive = false;
            animator.GetComponent<HitboxController>().DisableHitbox();
        }
    }
}
```

### Sound Effect Triggers via State Callback

```csharp
public sealed class SoundOnStateBehaviour : StateMachineBehaviour
{
    [SerializeField] private AudioClip m_Clip;
    [SerializeField] private float m_Volume = 1f;

    public override void OnStateEnter(Animator animator, AnimatorStateInfo stateInfo, int layerIndex)
    {
        AudioSource source = animator.GetComponent<AudioSource>();
        if (source != null)
        {
            source.PlayOneShot(m_Clip, m_Volume);
        }
    }
}
```

### Avoiding Allocations in State Behaviours

- Cache component references. `GetComponent<T>` in `OnStateEnter` is acceptable (called once per state entry), but never in `OnStateUpdate`.
- Do not use LINQ or create collections inside callbacks.
- StateMachineBehaviour instances are shared across Animator clones when using `RuntimeAnimatorController` — avoid storing per-instance state that differs between characters. Use the `animator` parameter to access per-object data instead.

## Combo System Pattern

Chain attacks using Animator state transitions with timed input windows.

### Animator Setup

```
Entry → Idle
Idle → Attack1 (trigger: Attack)
Attack1 → Attack2 (trigger: Attack, Has Exit Time: true, Exit Time: 0.6)
Attack2 → Attack3 (trigger: Attack, Has Exit Time: true, Exit Time: 0.6)
Attack3 → Idle (Has Exit Time: true)
Attack1 → Idle (Has Exit Time: true, no conditions — timeout fallback)
Attack2 → Idle (Has Exit Time: true, no conditions — timeout fallback)
```

### ComboSystem Implementation

```csharp
public sealed class ComboSystem
{
    private static readonly int k_AttackTrigger = Animator.StringToHash("Attack");

    private readonly Animator m_Animator;
    private int m_ComboStep;
    private float m_ComboResetTimer;
    private bool m_InputBuffered;

    private const float k_ComboWindowDuration = 0.8f;
    private const int k_MaxComboSteps = 3;

    public ComboSystem(Animator animator)
    {
        m_Animator = animator;
    }

    public void OnAttackInput()
    {
        if (m_ComboStep == 0)
        {
            // Start combo
            m_Animator.SetTrigger(k_AttackTrigger);
            m_ComboStep = 1;
            m_ComboResetTimer = k_ComboWindowDuration;
            m_InputBuffered = false;
        }
        else if (m_ComboStep < k_MaxComboSteps)
        {
            // Buffer the next attack — consumed when current animation reaches exit time
            m_InputBuffered = true;
        }
    }

    public void Tick(float deltaTime)
    {
        if (m_ComboStep == 0)
        {
            return;
        }

        m_ComboResetTimer -= deltaTime;

        if (m_ComboResetTimer <= 0f)
        {
            ResetCombo();
            return;
        }

        if (m_InputBuffered)
        {
            m_InputBuffered = false;
            m_Animator.SetTrigger(k_AttackTrigger);
            m_ComboStep++;
            m_ComboResetTimer = k_ComboWindowDuration;
        }
    }

    public void ResetCombo()
    {
        m_ComboStep = 0;
        m_InputBuffered = false;
        m_ComboResetTimer = 0f;
    }
}
```

The View calls `OnAttackInput()` from the input callback. `Tick()` runs from Update. When the current attack animation reaches its exit time window, the buffered trigger advances to the next combo state.

## Animator Performance

### Culling Modes

Set `Animator.cullingMode` to control what happens when the character is off-screen:
- **AlwaysAnimate:** full evaluation even off-screen. Use for gameplay-critical animations (AI that must keep walking).
- **CullUpdateTransforms:** evaluates the state machine but skips writing transforms. Keeps state accurate, saves transform cost.
- **CullCompletely:** stops evaluation entirely off-screen. Most performant. Use for background NPCs. Beware: state resets when returning to view.

```csharp
// Set in Awake or when LOD changes
m_Animator.cullingMode = AnimatorCullingMode.CullUpdateTransforms;
```

### Keep Animator State on Disable

```csharp
// Prevent state machine reset when pooling (SetActive toggle)
m_Animator.keepAnimatorStateOnDisable = true;
```

This is critical for object pooling. Without it, re-enabling the Animator resets to the entry state and replays Awake transitions.

### LOD-Based Animator Simplification

For distant characters, reduce animation cost:

```csharp
// Reduce bone evaluation at distance
public void SetAnimatorLOD(int lodLevel)
{
    switch (lodLevel)
    {
        case 0: // Close — full quality
            m_Animator.cullingMode = AnimatorCullingMode.AlwaysAnimate;
            m_Animator.speed = 1f;
            break;
        case 1: // Medium — skip transforms when off-screen
            m_Animator.cullingMode = AnimatorCullingMode.CullUpdateTransforms;
            m_Animator.speed = 1f;
            break;
        case 2: // Far — cull completely, half-rate update
            m_Animator.cullingMode = AnimatorCullingMode.CullCompletely;
            break;
    }
}
```

### Update Mode Selection

Set `Animator.updateMode` based on context:
- **Normal:** updates in `Update()`. Standard for most characters.
- **AnimatePhysics:** updates in `FixedUpdate()`. Use when root motion drives a Rigidbody.
- **UnscaledTime:** ignores `Time.timeScale`. Use for UI animations and pause menus.

```csharp
// Physics-driven root motion character
m_Animator.updateMode = AnimatorUpdateMode.AnimatePhysics;

// UI animation that plays during pause
m_Animator.updateMode = AnimatorUpdateMode.UnscaledTime;
```

## Animation Events Integration

### Footstep Sound via Animation Events

Add animation events at each foot-plant frame in the walk/run clips. The event calls a receiver on the same GameObject:

```csharp
public sealed class AnimationEventReceiver : MonoBehaviour
{
    [SerializeField] private AudioClip[] m_FootstepClips;
    [SerializeField] private AudioSource m_AudioSource;

    // Called by animation event — method name matches event function name
    public void OnFootstep(int footIndex)
    {
        if (m_FootstepClips.Length == 0)
        {
            return;
        }

        int clipIndex = footIndex % m_FootstepClips.Length;
        m_AudioSource.PlayOneShot(m_FootstepClips[clipIndex]);
    }
}
```

### VFX Spawn Timing

Place animation events at impact frames to spawn VFX at the exact moment of contact. The event forwards to a VFX controller that pulls from an object pool.

### Event Receiver Forwarding Pattern

Keep MonoBehaviour event receivers thin. Forward to injected systems:

```csharp
public sealed class AnimEventForwarder : MonoBehaviour
{
    private ICombatSystem m_CombatSystem;
    private IAudioSystem m_AudioSystem;

    [Inject]
    public void Construct(ICombatSystem combatSystem, IAudioSystem audioSystem)
    {
        m_CombatSystem = combatSystem;
        m_AudioSystem = audioSystem;
    }

    public void OnHitFrame() => m_CombatSystem.ProcessHitFrame();
    public void OnFootstep(int foot) => m_AudioSystem.PlayFootstep(foot);
    public void OnVFXSpawn(string vfxId) => m_CombatSystem.SpawnVFX(vfxId);
}
```

### Avoiding String-Based Event Names

Animation events use method names as strings, which cannot be validated at compile time. Mitigate this by:
- Keeping all event handler methods in a single `AnimEventForwarder` component per character
- Using a consistent naming convention: `On` + action name (`OnFootstep`, `OnHitFrame`, `OnVFXSpawn`)
- Never renaming event methods without updating every animation clip that references them
- Adding `#if UNITY_EDITOR` validation that checks event method existence on the target component
