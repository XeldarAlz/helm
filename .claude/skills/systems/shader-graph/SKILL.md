---
name: shader-graph
description: "ShaderGraph — custom function nodes, sub-graphs, keyword-driven variants, master stack outputs, common patterns for URP effects."
globs: ["**/*.shadergraph", "**/*.shadersubgraph"]
---

# ShaderGraph

## Overview

ShaderGraph is Unity's visual shader editor. Node-based, works with URP and HDRP. Generates HLSL shader code from the graph.

## Master Stack Outputs

### Vertex Stage
- Position (object/world/absolute world)
- Normal (object/tangent)

### Fragment Stage (URP Lit)
- Base Color, Normal (Tangent), Metallic, Smoothness, Emission, Ambient Occlusion, Alpha

### Fragment Stage (URP Unlit)
- Base Color, Alpha

## Custom Function Nodes

### Inline (small functions)
```hlsl
// In Custom Function node, Type: String
void MyFunction_float(float3 In, out float3 Out)
{
    Out = In * 2.0;
}
```

### External File (complex functions)
Create `.hlsl` file in project:
```hlsl
// Assets/Shaders/MyFunctions.hlsl
void TriplanarMapping_float(
    float3 Position, float3 Normal, float Sharpness,
    UnityTexture2D Tex, UnitySamplerState Sampler,
    out float4 Color)
{
    float3 blend = pow(abs(Normal), Sharpness);
    blend /= dot(blend, 1.0);

    float4 xProj = SAMPLE_TEXTURE2D(Tex, Sampler, Position.yz);
    float4 yProj = SAMPLE_TEXTURE2D(Tex, Sampler, Position.xz);
    float4 zProj = SAMPLE_TEXTURE2D(Tex, Sampler, Position.xy);

    Color = xProj * blend.x + yProj * blend.y + zProj * blend.z;
}
```

Reference in Custom Function node: Source = Asset, File = MyFunctions.hlsl

## Keywords (Shader Variants)

- **Boolean Keyword:** toggle features on/off per material
- **Enum Keyword:** select between N options
- Use `shader_feature` (stripped if unused) not `multi_compile` (always included)
- Use `shader_feature_local` for material-only keywords

Keep total variant count **under 1000** per shader.

## Common Patterns

### Dissolve Effect
1. Sample noise texture (Gradient Noise or texture)
2. Compare noise value to "Dissolve Amount" property (Step or SmoothStep)
3. Multiply with Alpha output
4. Add emission at dissolve edge (edge = small range above threshold)

### Fresnel / Rim Lighting
1. Fresnel Effect node (View Direction, Normal)
2. Multiply by color
3. Add to Emission

### Scrolling UV (Water, Lava)
1. Time node → Multiply by scroll speed
2. Add to UV coordinates
3. Sample texture with modified UVs

### Vertex Displacement (Wind, Waves)
1. Object Position + Time → noise function
2. Multiply by displacement amount
3. Add to Vertex Position output

### Outline (Inverted Hull Method)
Two-pass: Pass 1 = normal render, Pass 2 = vertex-expanded back faces with solid color.
(Requires custom Renderer Feature in URP or ShaderGraph with two materials.)

## Sub-Graphs

Reusable node groups. Create for common operations:
- Triplanar mapping
- Tiling and offset with rotation
- Blend modes (overlay, multiply, screen)
- Parallax mapping

## Performance Tips

- Minimize texture samples per fragment
- Use `half` precision where possible (set in graph settings)
- Avoid branching (use lerp/step instead)
- Fewer keywords = fewer variants = faster build times
- Preview variant count in Shader Inspector

## Custom Functions Deep Dive

### External .hlsl File Integration

Place custom HLSL files in `Assets/Shaders/Include/`. Reference them in Custom Function
nodes with Source set to "Asset". The function name must end with `_float` or `_half`
to match the precision selected in the graph.

```hlsl
// Assets/Shaders/Include/NoiseUtils.hlsl
// Voronoi noise — returns cell distance and cell ID
void Voronoi_float(float2 UV, float CellDensity, float AngleOffset,
    out float Distance, out float CellID)
{
    float2 cell = floor(UV * CellDensity);
    float2 frac = frac(UV * CellDensity);
    float minDist = 1.0;
    float id = 0.0;

    for (int y = -1; y <= 1; y++)
    {
        for (int x = -1; x <= 1; x++)
        {
            float2 neighbor = float2(x, y);
            float2 randomOffset = frac(sin(dot(cell + neighbor, float2(127.1, 311.7))) * 43758.5453);
            randomOffset = 0.5 + 0.5 * sin(AngleOffset + 6.2831 * randomOffset);
            float2 diff = neighbor + randomOffset - frac;
            float dist = dot(diff, diff);
            if (dist < minDist)
            {
                minDist = dist;
                id = dot(cell + neighbor, float2(1.0, 133.0));
            }
        }
    }
    Distance = sqrt(minDist);
    CellID = frac(id / 100.0);
}
```

### Texture2DArray Sampling

Sample from a Texture2DArray using a custom function node with an index parameter:

```hlsl
// Assets/Shaders/Include/ArraySampling.hlsl
void SampleTextureArray_float(UnityTexture2DArray TexArray, UnitySamplerState Sampler,
    float2 UV, float Index, out float4 Color)
{
    Color = SAMPLE_TEXTURE2D_ARRAY(TexArray, Sampler, UV, Index);
}
```

### Normal Reconstruction from Height Map

Convert a grayscale height map to a normal map inside the shader:

```hlsl
// Assets/Shaders/Include/NormalFromHeight.hlsl
void NormalFromHeight_float(UnityTexture2D HeightTex, UnitySamplerState Sampler,
    float2 UV, float Strength, float4 TexelSize, out float3 Normal)
{
    float left  = SAMPLE_TEXTURE2D(HeightTex, Sampler, UV - float2(TexelSize.x, 0)).r;
    float right = SAMPLE_TEXTURE2D(HeightTex, Sampler, UV + float2(TexelSize.x, 0)).r;
    float down  = SAMPLE_TEXTURE2D(HeightTex, Sampler, UV - float2(0, TexelSize.y)).r;
    float up    = SAMPLE_TEXTURE2D(HeightTex, Sampler, UV + float2(0, TexelSize.y)).r;

    float3 n;
    n.x = (left - right) * Strength;
    n.y = (down - up) * Strength;
    n.z = 1.0;
    Normal = normalize(n);
}
```

### Passing Custom Data via TexelSize

Unity auto-populates `_TextureName_TexelSize` as `float4(1/width, 1/height, width, height)`.
Access it in custom functions by declaring a `float4 TexelSize` parameter and connecting
the Texel Size output from a Texture2D property node.

## Keyword and Variant Management

### shader_feature vs multi_compile

| Directive | Behavior | Use When |
|-----------|----------|----------|
| `shader_feature` | Strips unused variants from build | Per-material toggles (most keywords) |
| `multi_compile` | Includes all variants in build | Global keywords set from code (fog, lightmap) |

In ShaderGraph, set the keyword "Definition" dropdown:
- **Shader Feature** — strips unused, smaller builds
- **Multi Compile** — always includes, needed if set globally at runtime

### Local Keywords (Per-Material)

Local keywords are scoped to a single shader, not the global keyword limit (currently 384).
In ShaderGraph, check "Exposed" and set Scope to "Local":

```
Keyword: _DETAIL_MAP
  Definition: Shader Feature
  Scope: Local
  Default: Off
```

Each material stores its own on/off state. Saves variant memory compared to global keywords.

### Variant Stripping in Build

Implement `IPreprocessShaders` to strip unused variants at build time:

```csharp
#if UNITY_EDITOR
using System.Collections.Generic;
using UnityEditor.Build;
using UnityEditor.Rendering;
using UnityEngine;
using UnityEngine.Rendering;

public sealed class ShaderVariantStripper : IPreprocessShaders
{
    public int callbackOrder => 0;

    public void OnProcessShader(Shader shader, ShaderSnippetData snippet,
        IList<ShaderCompilerData> data)
    {
        ShaderKeyword fogKeyword = new ShaderKeyword("FOG_EXP2");
        for (int variantIndex = data.Count - 1; variantIndex >= 0; variantIndex--)
        {
            if (data[variantIndex].shaderKeywordSet.IsEnabled(fogKeyword))
            {
                data.RemoveAt(variantIndex);
            }
        }
    }
}
#endif
```

### Boolean Keyword for Simple Toggles

Boolean keywords generate exactly 2 variants (on/off). Use them for feature toggles
like emission, detail map, or vertex color. In ShaderGraph:

1. Add Keyword node (Boolean)
2. Connect True/False outputs to the relevant branch
3. The keyword appears as a checkbox on the material inspector

## Lighting Integration

### Main Light Node in URP

The Main Light Direction node provides sun/directional light data in the fragment shader.
Use it for custom lighting models:

```
[Main Light Direction] → dot with Normal → Saturate → custom ramp or step
```

For full main light data (color, attenuation, shadow), use a custom function:

```hlsl
// Assets/Shaders/Include/MainLightData.hlsl
void GetMainLight_float(float3 WorldPosition, out float3 Direction,
    out float3 Color, out float Attenuation)
{
    Light mainLight = GetMainLight(TransformWorldToShadowCoord(WorldPosition));
    Direction = mainLight.direction;
    Color = mainLight.color;
    Attenuation = mainLight.distanceAttenuation * mainLight.shadowAttenuation;
}
```

### Shadow Receiving in Custom Shaders

To receive shadows in a ShaderGraph shader:
1. Set Surface Type to Opaque or Transparent with "Receive Shadows" checked
2. For custom lighting, sample the shadow map via the Main Light custom function above
3. Multiply your final color by `Attenuation` to apply shadows

### Normal Map Application (Tangent Space)

Connect a normal map sample to the Fragment Normal (Tangent) output:

```
[Sample Texture 2D] (Normal Map, type: Normal) → [Normal Strength] → Fragment Normal (Tangent)
```

Set the texture type to "Normal" in the Sample Texture 2D node. The Normal Strength
node controls intensity (1 = full, 0 = flat).

### Ambient Color and Probe Sampling

Sample ambient light and reflection probes for indirect lighting:

```hlsl
// Assets/Shaders/Include/AmbientSampling.hlsl
void SampleAmbient_float(float3 WorldNormal, out float3 Ambient)
{
    Ambient = SampleSH(WorldNormal);
}

void SampleReflectionProbe_float(float3 WorldReflection, float Roughness,
    out float3 Reflection)
{
    Reflection = GlossyEnvironmentReflection(WorldReflection, Roughness, 1.0);
}
```

## Advanced Effects Library

### Toon / Cel Shading

Quantize lighting into discrete steps for a cartoon look:

```
Graph setup:
1. [Main Light Direction] → Dot Product with [World Normal]
2. Saturate the result (0 to 1 range)
3. Multiply by number of steps (e.g., 4)
4. Floor the result
5. Divide by number of steps
6. Multiply with Base Color → Fragment Base Color

For ramp texture approach:
1. Dot Product result → U coordinate of a 1D ramp texture
2. Sample ramp texture (set wrap mode to Clamp)
3. Multiply ramp color with Base Color
```

Ramp textures give artistic control: paint bands of color and shadow hue shifts.

### Hologram Effect

Combine scanlines, vertex offset, and alpha flicker:

```
Scanlines:
1. [Screen Position] → take Y component
2. Multiply by scanline density (e.g., 200)
3. Sine or Frac → Step at 0.5 threshold
4. Multiply into Alpha

Vertex Jitter:
1. [Time] → Multiply by jitter speed
2. Floor to snap to discrete frames (retro look)
3. Random Range or noise per-vertex
4. Small offset on X/Z → add to Vertex Position

Alpha Flicker:
1. [Time] → Sine with high frequency
2. Remap from (-1,1) to (0.3, 1.0)
3. Multiply into Alpha

Final: Set Surface Type = Transparent, Blend = Additive
```

### Water Surface

Scrolling normals with depth-based transparency and foam edges:

```
Normal Scrolling:
1. [UV] + [Time] * scroll speed A → Sample Normal Map A
2. [UV] + [Time] * scroll speed B (different direction) → Sample Normal Map B
3. Blend normals: normalize(A + B)
4. Connect to Fragment Normal (Tangent)

Depth Fade:
1. [Scene Depth] - [Fragment Depth] → depth difference
2. Saturate(depthDiff / fadeDistance)
3. Lerp between shallow color and deep color
4. Connect to Base Color

Foam Edge:
1. Same depth difference
2. Step or SmoothStep at foam threshold
3. Add foam color * foam mask to Emission

Vertex Waves:
1. [Object Position].xz + [Time] → Gradient Noise
2. Multiply by wave height
3. Add to Vertex Position Y
```

### Dissolve with Edge Glow

Noise-based dissolve with emission at the burn edge:

```
1. Sample noise texture (Gradient Noise or authored texture)
2. Subtract dissolve amount (0 = solid, 1 = fully dissolved)
3. Clip/Alpha Clip: connect noise - dissolve to Alpha, set threshold to 0

Edge Glow:
1. SmoothStep(dissolveAmount, dissolveAmount + edgeWidth, noiseValue)
2. One minus the result → edge mask (1 at the edge, 0 elsewhere)
3. Multiply edge mask by glow color
4. Add to Emission

Set Alpha Clipping ON in graph settings. Control dissolve amount from C# via
material.SetFloat("_DissolveAmount", value).
```

## Sub-Graph Patterns

### Reusable Sub-Graph Conventions

Structure sub-graphs as single-purpose utility blocks:
- Name with the pattern `SG_PurposeName` (e.g., `SG_TriplanarMapping`)
- Expose only essential inputs, use sensible defaults for the rest
- Document input ranges in the sub-graph description field
- Output common types (float4 for color, float3 for normal, float for mask)

### Triplanar Mapping Sub-Graph

Inputs: Texture2D, Sampler, World Position, World Normal, Blend Sharpness
Output: float4 Color

Internally samples the texture three times (XY, XZ, YZ planes) and blends
by the absolute world normal raised to the sharpness power. See the external
HLSL function in the Custom Functions section above.

### Fresnel + Rim Light Sub-Graph

Inputs: Normal (World), View Direction (World), Power, Color
Output: float3 Emission

```
1. [Fresnel Effect] node with Normal and View Dir inputs, Power parameter
2. Multiply fresnel result by Color
3. Output as Emission contribution
```

Use this sub-graph on any material that needs rim highlighting — characters,
pickups, interactable objects.

### Sub-Graph Input/Output Conventions

- Color inputs: default to white (1,1,1,1)
- Strength/power inputs: default to 1.0, range annotation in description
- Normal inputs: expect world space unless labeled "Tangent"
- UV inputs: default to UV0 channel if not connected
- Always provide a "Strength" or "Blend" input (0-1) so the effect can be faded

## Performance Optimization

### Half vs Float Precision

| Use Half (16-bit) | Use Float (32-bit) |
|--------------------|--------------------|
| Colors (0-1 range) | World positions |
| UV coordinates (small textures) | Large UV offsets (scrolling over time) |
| Normal vectors | Depth calculations |
| Most intermediate math | Screen-space coordinates |

Set precision in ShaderGraph: Graph Settings > Precision. Override per-node
when a specific calculation needs higher accuracy.

### Instruction Count Monitoring

Check compiled instruction count in the shader inspector:
- Vertex: aim for under 100 instructions for mobile
- Fragment: aim for under 200 instructions for mobile
- Desktop: double these budgets is acceptable

Reduce instructions by:
- Replacing complex math with lookup textures (LUT)
- Using `step()` instead of `if` branching
- Combining multiple simple operations into fewer complex ones

### Texture Sampling Reduction

Pack multiple grayscale maps into a single RGBA texture:
- R = Metallic
- G = Ambient Occlusion
- B = Detail Mask
- A = Smoothness

This turns 4 texture samples into 1. Split channels with a Swizzle or Split node.

### LOD-Based Shader Simplification

Use shader LOD to simplify materials at distance:
- LOD 0 (close): full shader with normal maps, detail textures, parallax
- LOD 1 (mid): drop parallax and detail textures
- LOD 2 (far): simple diffuse only, no normal map

Implement with keyword toggles controlled from a LOD Group callback,
or use separate ShaderGraph materials per LOD mesh.

```csharp
// Set shader LOD from a LOD callback on the renderer
public sealed class ShaderLodController : MonoBehaviour
{
    [SerializeField] private Renderer m_Renderer;
    [SerializeField] private Material m_FullMaterial;
    [SerializeField] private Material m_SimpleMaterial;

    private static readonly int k_DetailOn = Shader.PropertyToID("_DETAIL_ON");

    public void OnLodChanged(int lodLevel)
    {
        if (lodLevel >= 2)
        {
            m_Renderer.sharedMaterial = m_SimpleMaterial;
        }
        else
        {
            m_Renderer.sharedMaterial = m_FullMaterial;
            m_FullMaterial.SetFloat(k_DetailOn, lodLevel == 0 ? 1f : 0f);
        }
    }
}
```
