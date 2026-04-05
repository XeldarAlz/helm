---
name: cinemachine
description: "Cinemachine camera system — virtual cameras, FreeLook, blending, noise profiles, state-driven cameras, confiner, follow/aim behaviors."
globs: ["**/*Cinemachine*", "**/*Camera*.cs", "**/*Cam*.cs"]
---

# Cinemachine

## Setup

1. Add `CinemachineBrain` component to Main Camera
2. Create Virtual Cameras — Brain auto-blends to highest priority

## Virtual Camera Components

**Body (follow):**
- `Transposer` — 3D offset follow (configurable damping)
- `Framing Transposer` — 2D/screen-space follow (dead zone, soft zone)
- `Orbital Transposer` — orbit around target (user input rotates)
- `Tracked Dolly` — follow a path

**Aim (look at):**
- `Composer` — keep target in frame with dead/soft zones
- `Group Composer` — frame a group of targets
- `Hard Look At` — no damping, instant look
- `POV` — player-controlled rotation (FPS)

## Common Setups

### 2D Platformer Camera
```
Virtual Camera:
  Body: Framing Transposer
    - Screen X/Y: 0.5 (center)
    - Dead Zone Width: 0.1, Height: 0.1
    - Damping: X=1, Y=0.5
  Follow: Player Transform
  Add Extension: CinemachineConfiner2D
    - Bounding Shape: PolygonCollider2D (room bounds)
```

### 3D Third-Person Camera
```
FreeLook Camera:
  Follow: Player Transform
  Look At: Player Head/Chest
  Top/Middle/Bottom Rig:
    - Height, Radius per rig
    - Composer aim in each rig
  X Axis: Input from mouse/stick (orbiting)
  Y Axis: Input from mouse/stick (elevation)
```

### State-Driven Camera (Animator)
```
State-Driven Camera:
  Animated Target: Player Animator
  States:
    Idle → VCam_Idle (wide shot)
    Run → VCam_Run (further back)
    Combat → VCam_Combat (over shoulder)
```

## Camera Blending

- Default blend: 2 seconds, EaseInOut
- Custom blends per transition (from VCam A to VCam B)
- Cut (0 seconds) for instant switches

## Cinemachine Impulse (Screen Shake)

```csharp
// Source: generates impulse
[SerializeField] private CinemachineImpulseSource m_ImpulseSource;

public void OnExplosion()
{
    m_ImpulseSource.GenerateImpulse();
}
```

Add `CinemachineImpulseListener` extension to virtual cameras that should respond.

## Noise (Handheld Feel)

Add `CinemachineBasicMultiChannelPerlin` to virtual camera:
- Profile: `6D Shake` or `Handheld_normal_mild`
- Amplitude/Frequency for intensity

## Code Control

```csharp
// Switch cameras by priority
m_CombatCamera.Priority = 20; // Higher = active
m_ExploreCamera.Priority = 10;

// Change follow target
m_VirtualCamera.Follow = newTarget;
m_VirtualCamera.LookAt = newTarget;
```

## Confiner

- **2D:** `CinemachineConfiner2D` + `PolygonCollider2D` (set collider to trigger, non-physics layer)
- **3D:** `CinemachineConfiner` + `BoxCollider` or `MeshCollider` volume

## Follow and Aim Behaviors Deep Dive

### Damping Tuning by Genre

Damping controls how quickly the camera follows the target. Lower values mean snappier response, higher values add lag for a smoother feel.

**2D Platformer:**
- X Damping: 0.5 to 1.0 (responsive horizontal tracking)
- Y Damping: 1.5 to 3.0 (slow vertical response to avoid jitter on jumps)
- Y Damping when target is below camera: 0.5 (fast follow on falls)

**Third Person Action:**
- X/Y/Z Damping: 1.0 to 2.0 (balanced, smooth follow)
- Yaw Damping: 0.5 (snappy rotation to keep combat targets framed)

**Top-Down / RTS:**
- All Damping: 0.0 to 0.3 (near-instant, camera is tool not cinematic element)

**Driving / Racing:**
- X/Z Damping: 2.0 to 4.0 (heavy lag creates speed sensation)
- Y Damping: 0.5 (keep road in frame)

### Lookahead Time and Smoothing

Lookahead shifts the camera ahead of the target's velocity so the player sees more of what is coming.

- `Lookahead Time`: 0.3 to 0.7 seconds for platformers, 0 for combat games
- `Lookahead Smoothing`: 5 to 15 — higher values smooth out jitter but add latency
- Set lookahead to 0 when the character changes direction rapidly (fighting games) to avoid oscillation

### Soft Zone vs Dead Zone

- **Dead Zone**: area where the target can move without camera adjustment. Larger dead zone = less camera movement = calmer feel. Good for exploration.
- **Soft Zone**: area outside the dead zone where the camera accelerates to catch up. Larger soft zone = more tolerance before hard tracking kicks in.
- For a responsive shooter: Dead Zone 0.02, Soft Zone 0.3
- For a relaxed adventure: Dead Zone 0.15, Soft Zone 0.6
- For a precision platformer: Dead Zone 0.05, Soft Zone 0.4

### Framing Transposer vs Transposer Selection

| Need | Choose |
|------|--------|
| 2D game or side-scroller | Framing Transposer — works in screen space |
| 3D game with offset follow | Transposer — works in world space |
| Camera must maintain screen-space framing | Framing Transposer |
| Camera offset defined in meters from target | Transposer |
| Need dead zone and soft zone in screen space | Framing Transposer |
| Camera position relative to target regardless of screen size | Transposer |

## Advanced Blending

### Custom Blend Curves

Create a `CinemachineBlenderSettings` asset (Assets > Create > Cinemachine > Blender Settings) to define per-transition blends.

Assign the asset to the `CinemachineBrain.CustomBlends` field.

Each entry specifies: From Camera, To Camera, Blend Style, and Blend Time.

**Blend styles:**
- `EaseInOut` — smooth acceleration and deceleration, best default
- `EaseIn` — soft start, abrupt end, good for entering action
- `EaseOut` — abrupt start, soft end, good for settling into a scene
- `Linear` — constant speed, useful for mechanical/UI cameras
- `HardIn` — fast start, used for impact moments
- `HardOut` — fast end, used to snap into position
- `Custom` — provide your own AnimationCurve for full control

### Cut vs Blend Decision

- Cut (blend time 0) when the cameras share no spatial relationship (teleport, scene change, different room)
- Blend (0.5 to 2.0 seconds) when cameras are in the same space and the player should perceive continuous movement
- Short blend (0.3 seconds) for gameplay-critical transitions (aiming down sights)
- Long blend (2.0+ seconds) for cinematic transitions (cutscene entry)

### Blend-to-Latest vs Blend-to-Highest Priority

On `CinemachineBrain`, the `DefaultBlend` applies when no custom blend exists.

When multiple cameras activate simultaneously:
- Default behavior blends to the highest priority camera
- If two cameras share priority, the most recently activated one wins
- Use `CinemachineBlendListCamera` to sequence multiple cameras in a scripted order

### Blend Interruption

When a blend is in progress and a new camera activates:
- The brain starts a new blend from the current blended state
- This creates smooth transitions even during rapid camera changes
- Avoid activating cameras every frame — it resets the blend and causes jitter

## State-Driven Camera System

### Complete Setup Example

```csharp
using UnityEngine;
using Unity.Cinemachine;
using VContainer;

// Camera mode enum shared between systems
public enum CameraMode
{
    Exploration,
    Combat,
    Dialogue
}

// System that manages camera state transitions
public sealed class CameraStateSystem : System.IDisposable
{
    private readonly CinemachineCamera m_ExplorationCam;
    private readonly CinemachineCamera m_CombatCam;
    private readonly CinemachineCamera m_DialogueCam;

    private const int k_ActivePriority = 20;
    private const int k_InactivePriority = 0;

    private CameraMode m_CurrentMode = CameraMode.Exploration;

    [Inject]
    public CameraStateSystem(
        [Inject(Id = "ExplorationCam")] CinemachineCamera explorationCam,
        [Inject(Id = "CombatCam")] CinemachineCamera combatCam,
        [Inject(Id = "DialogueCam")] CinemachineCamera dialogueCam)
    {
        m_ExplorationCam = explorationCam;
        m_CombatCam = combatCam;
        m_DialogueCam = dialogueCam;

        SetMode(CameraMode.Exploration);
    }

    public void SetMode(CameraMode mode)
    {
        m_CurrentMode = mode;

        m_ExplorationCam.Priority = mode == CameraMode.Exploration ? k_ActivePriority : k_InactivePriority;
        m_CombatCam.Priority = mode == CameraMode.Combat ? k_ActivePriority : k_InactivePriority;
        m_DialogueCam.Priority = mode == CameraMode.Dialogue ? k_ActivePriority : k_InactivePriority;
    }

    public CameraMode CurrentMode => m_CurrentMode;

    public void Dispose() { }
}
```

### Animator-Driven Camera Switching

Use `CinemachineStateDrivenCamera` when camera changes map directly to animation states.

```
CinemachineStateDrivenCamera:
  Animated Target: Character Animator
  Default Blend: EaseInOut 1.0s
  State-Camera Pairs:
    "Idle"    → VCam_Idle    (wide, slow follow)
    "Run"     → VCam_Run     (pulled back, higher damping)
    "Attack"  → VCam_Attack  (over-shoulder, tight framing)
    "Hurt"    → VCam_Hurt    (slight zoom, screen shake via noise)
    "Death"   → VCam_Death   (slow dolly out, cut blend)
```

Each child camera can have its own body, aim, noise, and extensions. The state-driven parent handles activation based on animator state.

### Code-Driven Priority Switching

For systems that do not map cleanly to animator states, manage priorities directly.

```csharp
// Reacting to a MessagePipe message to switch camera
public sealed class CombatCameraResponder : System.IDisposable
{
    private readonly CameraStateSystem m_CameraState;
    private readonly System.IDisposable m_Subscription;

    [Inject]
    public CombatCameraResponder(
        CameraStateSystem cameraState,
        ISubscriber<CombatEnteredMessage> combatEntered)
    {
        m_CameraState = cameraState;
        m_Subscription = combatEntered.Subscribe(OnCombatEntered);
    }

    private void OnCombatEntered(CombatEnteredMessage message)
    {
        m_CameraState.SetMode(CameraMode.Combat);
    }

    public void Dispose() => m_Subscription.Dispose();
}
```

## Advanced Impulse System

### Multiple Impulse Channels

Use `CinemachineImpulseChannels` to separate different shake sources. Each source and listener has a channel mask (bitmask). A listener only responds to impulses on matching channels.

```
Channel 1: Combat (weapon impacts, explosions)
Channel 2: Environment (earthquakes, collapsing structures)
Channel 3: UI Feedback (menu confirm, error shake)
```

Configure channel mask on both `CinemachineImpulseSource` and `CinemachineImpulseListener` to isolate which cameras react to which events.

### Impulse Profiles

**Instant impulse (explosion, hit):**
- Raw Signal: `6D Shake`
- Time Envelope: Attack 0.0, Sustain 0.05, Decay 0.3
- Amplitude: 1.0 to 3.0 based on damage/force

**Sustained impulse (earthquake, engine rumble):**
- Raw Signal: `Handheld_normal_mild`
- Time Envelope: Attack 0.5, Sustain 2.0, Decay 1.0
- Amplitude: 0.3 to 0.8 (lower amplitude, longer duration)

**Directional impulse (recoil):**
- Use `CinemachineImpulseSource.GenerateImpulse(Vector3 velocity)` to specify direction
- Velocity magnitude controls amplitude, direction controls shake bias

### Amplitude and Frequency Scaling by Distance

```csharp
public sealed class DistanceScaledImpulse : MonoBehaviour
{
    [SerializeField] private CinemachineImpulseSource m_ImpulseSource;
    [SerializeField] private float m_MaxDistance = 30f;
    [SerializeField] private float m_MaxAmplitude = 2f;

    public void TriggerAtPosition(Vector3 sourcePosition, Vector3 listenerPosition)
    {
        float distance = Vector3.Distance(sourcePosition, listenerPosition);
        float normalizedDistance = Mathf.Clamp01(distance / m_MaxDistance);

        // Inverse-square falloff
        float amplitude = m_MaxAmplitude * (1f - normalizedDistance * normalizedDistance);

        if (amplitude > 0.01f)
        {
            m_ImpulseSource.GenerateImpulse(amplitude);
        }
    }
}
```

### Impulse Listener Filtering

On the `CinemachineImpulseListener` extension:
- `Gain`: multiplier on incoming impulse amplitude (0 = deaf, 1 = normal, 2 = amplified)
- `Use 2D Distance`: ignore Y axis for distance attenuation (good for side-scrollers)
- `Channel Mask`: bitmask to filter which impulse channels this listener reacts to
- Combine gain with per-camera settings for cinematic cameras that shake less than gameplay cameras

## FreeLook Camera

### Three-Rig Setup

A FreeLook camera uses three orbital rigs stacked vertically: Top, Middle, Bottom. The player orbits the character by moving the Y axis between rigs and the X axis around the character.

```
Top Rig:    Height 4.5, Radius 1.5  (bird's-eye, tight orbit)
Middle Rig: Height 2.5, Radius 4.0  (default gameplay view)
Bottom Rig: Height 0.4, Radius 4.5  (low angle, dramatic)
```

Each rig has its own Composer aim settings. The camera interpolates between rigs as the Y axis changes.

### Per-Rig Radius and Height Tuning

- **Top Rig**: small radius, high height — looking down on the character, good for seeing surroundings
- **Middle Rig**: medium radius, eye-level height — primary gameplay view, tune this first
- **Bottom Rig**: larger radius, low height — ground-level dramatic angle
- Spline tension (0 to 1) controls how smoothly the camera interpolates between rigs. Higher tension = rounder path.

### Input Axis Speed and Acceleration

```
X Axis (horizontal orbit):
  Max Speed: 300 (degrees per second with full stick deflection)
  Accel Time: 0.1 (seconds to reach max speed)
  Decel Time: 0.15 (seconds to stop)
  Invert: false (platform convention)

Y Axis (vertical rig blend):
  Max Speed: 2.0 (blend units per second, 0 = bottom rig, 1 = top rig)
  Accel Time: 0.1
  Decel Time: 0.15
  Invert: true (push up to look down is common in third-person)
```

### Recentering Behavior

Recentering automatically moves the camera behind the character when input stops.

- `Wait Time`: seconds of no input before recentering starts (1.0 to 3.0)
- `Recentering Time`: seconds to complete the recenter (0.5 to 2.0)
- Enable recentering on X axis for gameplay cameras
- Disable recentering on Y axis to let players keep their preferred elevation
- Disable recentering entirely during combat or aiming

## Dynamic Confiner

### Confiner2D with Runtime Polygon Update

```csharp
using UnityEngine;
using Unity.Cinemachine;

public sealed class RoomConfinerSwapper : MonoBehaviour
{
    [SerializeField] private CinemachineConfiner2D m_Confiner;

    public void SwitchToRoom(PolygonCollider2D roomBounds)
    {
        m_Confiner.BoundingShape2D = roomBounds;
        m_Confiner.InvalidateBoundingShapeCache();
    }
}
```

Call `InvalidateBoundingShapeCache()` whenever the bounding shape reference or geometry changes. Without this call the confiner uses stale data.

### Room-Based Confiner Switching

For games with discrete rooms (metroidvania, dungeon crawler):
1. Each room has a `PolygonCollider2D` on a dedicated "CameraBounds" layer (set as trigger)
2. When the player enters a room trigger, call `SwitchToRoom` with the new collider
3. The Cinemachine brain blend handles the smooth camera transition between rooms

### Smooth Confiner Transitions

By default, switching the confiner boundary causes a snap. To smooth it:
- Rely on the camera's own damping — if damping is high enough, the confiner switch blends naturally
- Alternatively, temporarily increase the `Damping` value on the confiner extension during the switch, then restore it
- For 2D: `CinemachineConfiner2D.Damping` controls how quickly the camera is pushed back inside bounds. A value of 1 to 3 provides a smooth pull rather than a hard clamp.

### Confiner Padding for UI Safe Area

Inset the confiner polygon so the camera does not push important content behind UI elements:
- Shrink the `PolygonCollider2D` points inward by the UI margin (e.g., 1 unit on each side)
- Or use a separate smaller collider specifically for camera bounds
- For dynamic UI (mobile notch, varying safe areas), adjust confiner bounds at runtime based on `Screen.safeArea`

## Camera Debugging

### Brain Debugging

Select the Main Camera (with `CinemachineBrain`) in Play mode to see:
- **Active Virtual Camera**: which camera currently controls the view
- **Active Blend**: shows "from" and "to" cameras with blend progress
- **Live Camera**: the camera currently being evaluated

Enable `CinemachineBrain.ShowDebugText` in the inspector to display the active camera name on screen during play mode.

### Solo Mode

In the Scene view, select a Virtual Camera and click the "Solo" button in the Cinemachine inspector. This forces the Game view to use that camera regardless of priority. Useful for tuning individual camera settings without triggering blend logic.

Remember to un-solo before testing transitions.

### Gizmo Interpretation

- **Yellow frustum**: the virtual camera's field of view and near/far planes
- **Red/green crosshairs**: dead zone (red) and soft zone (green) boundaries in the game view
- **Blue wireframe sphere**: follow target offset position
- **Path gizmos** (Tracked Dolly): the dolly path with waypoints and tangent handles
- **Confiner outline**: the bounding shape drawn in the scene view when the confiner extension is selected
