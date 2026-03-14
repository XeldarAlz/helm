<script lang="ts">
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { settings, type AppSettings } from "$lib/stores/settings";
  import { getSettings, updateSettings } from "$lib/utils/ipc";
  import { addToast } from "$lib/stores/toasts";
  import { activeView } from "$lib/stores/ui";
  import { staggeredItem } from "$lib/animations";
  import TopBar from "$lib/components/layout/TopBar.svelte";
  import Panel from "$lib/components/layout/Panel.svelte";
  import TextInput from "$lib/components/common/TextInput.svelte";
  import Dropdown from "$lib/components/common/Dropdown.svelte";
  import Slider from "$lib/components/common/Slider.svelte";
  import Toggle from "$lib/components/common/Toggle.svelte";
  import Button from "$lib/components/common/Button.svelte";
  import {
    FolderOpen,
    Terminal,
    Bot,
    Cpu,
    Shield,
    Palette,
    Keyboard,
    Info,
  } from "lucide-svelte";

  let local = $state<AppSettings>({
    projectDir: "",
    claudeCliPath: "claude",
    maxConcurrentAgents: 3,
    agentModels: { coder: "opus", tester: "sonnet", reviewer: "opus", unitySetup: "sonnet", committer: "haiku" },
    fontSize: 13,
    sidebarCollapsed: false,
    theme: "dark",
    reducedMotion: false,
  });
  let mounted = $state(false);
  let saving = $state(false);
  let dirty = $state(false);

  const modelOptions = [
    { value: "opus", label: "Claude Opus (Most capable)" },
    { value: "sonnet", label: "Claude Sonnet (Balanced)" },
    { value: "haiku", label: "Claude Haiku (Fastest)" },
  ];

  const agentKeys: { key: keyof AppSettings["agentModels"]; label: string; color: string }[] = [
    { key: "coder", label: "Coder", color: "var(--color-agent-coder)" },
    { key: "tester", label: "Tester", color: "var(--color-agent-tester)" },
    { key: "reviewer", label: "Reviewer", color: "var(--color-agent-reviewer)" },
    { key: "unitySetup", label: "Unity Setup", color: "var(--color-agent-unity)" },
    { key: "committer", label: "Committer", color: "var(--color-agent-commit)" },
  ];

  const shortcuts = [
    { keys: ["Cmd", "N"], action: "settings.shortcuts.newSession" },
    { keys: ["Cmd", "W"], action: "settings.shortcuts.closeSession" },
    { keys: ["Cmd", ","], action: "settings.shortcuts.openSettings" },
    { keys: ["Cmd", "1"], action: "settings.shortcuts.dashboard" },
    { keys: ["Cmd", "\\"], action: "settings.shortcuts.toggleSidebar" },
  ];

  function markDirty() {
    dirty = true;
  }

  async function save() {
    saving = true;
    try {
      // Map camelCase to snake_case for Rust
      await updateSettings({
        projectDir: local.projectDir,
        claudeCliPath: local.claudeCliPath,
        maxConcurrentAgents: local.maxConcurrentAgents,
        agentModels: local.agentModels,
        fontSize: local.fontSize,
        sidebarCollapsed: local.sidebarCollapsed,
        theme: local.theme,
        reducedMotion: local.reducedMotion,
      } as any);
      settings.set({ ...local });
      dirty = false;
      addToast($_("settings.saved"), "success");
    } catch (e) {
      addToast(`Failed to save: ${e}`, "error");
    } finally {
      saving = false;
    }
  }

  onMount(async () => {
    try {
      const loaded = await getSettings();
      // Normalize snake_case from Rust to camelCase
      local = {
        projectDir: (loaded as any).project_dir ?? (loaded as any).projectDir ?? "",
        claudeCliPath: (loaded as any).claude_cli_path ?? (loaded as any).claudeCliPath ?? "claude",
        maxConcurrentAgents: (loaded as any).max_concurrent_agents ?? (loaded as any).maxConcurrentAgents ?? 3,
        agentModels: {
          coder: (loaded as any).agent_models?.coder ?? (loaded as any).agentModels?.coder ?? "opus",
          tester: (loaded as any).agent_models?.tester ?? (loaded as any).agentModels?.tester ?? "sonnet",
          reviewer: (loaded as any).agent_models?.reviewer ?? (loaded as any).agentModels?.reviewer ?? "opus",
          unitySetup: (loaded as any).agent_models?.unity_setup ?? (loaded as any).agentModels?.unitySetup ?? "sonnet",
          committer: (loaded as any).agent_models?.committer ?? (loaded as any).agentModels?.committer ?? "haiku",
        },
        fontSize: (loaded as any).font_size ?? (loaded as any).fontSize ?? 13,
        sidebarCollapsed: (loaded as any).sidebar_collapsed ?? (loaded as any).sidebarCollapsed ?? false,
        theme: (loaded as any).theme ?? "dark",
        reducedMotion: (loaded as any).reduced_motion ?? (loaded as any).reducedMotion ?? false,
      };
      settings.set({ ...local });
    } catch {
      // Use defaults
    }
    mounted = true;
  });
</script>

<div class="flex flex-col h-full">
  <TopBar title={$_("settings.title")}>
    {#snippet actions()}
      {#if dirty}
        <Button variant="primary" size="sm" onclick={save} disabled={saving}>
          {saving ? $_("common.loading") : $_("common.save")}
        </Button>
      {/if}
    {/snippet}
  </TopBar>

  <div class="flex-1 overflow-y-auto">
    <div class="max-w-2xl mx-auto px-8 py-8 space-y-6">

      <!-- Project Section -->
      {#if mounted}
        <section in:staggeredItem={{ index: 0, staggerDelay: 60, duration: 250, distance: 8 }}>
          <Panel class="p-5">
            <div class="flex items-center gap-3 mb-4">
              <div class="flex items-center justify-center w-8 h-8 rounded-[var(--radius-md)] bg-[var(--color-accent)]/10">
                <FolderOpen size={16} class="text-[var(--color-accent)]" />
              </div>
              <div>
                <h3 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
                  {$_("settings.project")}
                </h3>
              </div>
            </div>

            <div class="space-y-4">
              <div>
                <label class="block text-[var(--text-caption)] text-[var(--color-text-secondary)] mb-1.5">
                  {$_("settings.projectDir")}
                </label>
                <TextInput
                  bind:value={local.projectDir}
                  placeholder="/path/to/your/unity/project"
                  oninput={markDirty}
                />
                <p class="text-[10px] text-[var(--color-text-tertiary)] mt-1">{$_("settings.projectDirDesc")}</p>
              </div>

              <div>
                <label class="block text-[var(--text-caption)] text-[var(--color-text-secondary)] mb-1.5">
                  {$_("settings.cliPath")}
                </label>
                <TextInput
                  bind:value={local.claudeCliPath}
                  placeholder="claude"
                  oninput={markDirty}
                />
                <p class="text-[10px] text-[var(--color-text-tertiary)] mt-1">{$_("settings.cliPathDesc")}</p>
              </div>
            </div>
          </Panel>
        </section>
      {/if}

      <!-- Agent Models Section -->
      {#if mounted}
        <section in:staggeredItem={{ index: 1, staggerDelay: 60, duration: 250, distance: 8 }}>
          <Panel class="p-5">
            <div class="flex items-center gap-3 mb-4">
              <div class="flex items-center justify-center w-8 h-8 rounded-[var(--radius-md)] bg-[var(--color-agent-coder)]/10">
                <Bot size={16} class="text-[var(--color-agent-coder)]" />
              </div>
              <div>
                <h3 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
                  {$_("settings.models")}
                </h3>
                <p class="text-[10px] text-[var(--color-text-tertiary)]">{$_("settings.modelsDesc")}</p>
              </div>
            </div>

            <div class="space-y-3">
              {#each agentKeys as agent}
                <div class="flex items-center gap-3">
                  <div class="w-3 h-3 rounded-full shrink-0" style="background: {agent.color}"></div>
                  <span class="text-[var(--text-body)] text-[var(--color-text-primary)] w-24 shrink-0">{agent.label}</span>
                  <Dropdown
                    options={modelOptions}
                    bind:value={local.agentModels[agent.key]}
                    onchange={markDirty}
                    class="flex-1"
                  />
                </div>
              {/each}
            </div>
          </Panel>
        </section>
      {/if}

      <!-- Parallelism Section -->
      {#if mounted}
        <section in:staggeredItem={{ index: 2, staggerDelay: 60, duration: 250, distance: 8 }}>
          <Panel class="p-5">
            <div class="flex items-center gap-3 mb-4">
              <div class="flex items-center justify-center w-8 h-8 rounded-[var(--radius-md)] bg-[var(--color-status-info)]/10">
                <Cpu size={16} class="text-[var(--color-status-info)]" />
              </div>
              <div>
                <h3 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
                  {$_("settings.parallelism")}
                </h3>
                <p class="text-[10px] text-[var(--color-text-tertiary)]">{$_("settings.maxAgentsDesc")}</p>
              </div>
            </div>

            <div class="flex items-center gap-4">
              <Slider
                bind:value={local.maxConcurrentAgents}
                min={1}
                max={8}
                step={1}
                onchange={markDirty}
                class="flex-1"
              />
              <span class="text-[var(--text-title)] font-semibold text-[var(--color-accent)] w-8 text-center">
                {local.maxConcurrentAgents}
              </span>
            </div>
          </Panel>
        </section>
      {/if}

      <!-- Hooks Section -->
      {#if mounted}
        <section in:staggeredItem={{ index: 3, staggerDelay: 60, duration: 250, distance: 8 }}>
          <Panel class="p-5">
            <div class="flex items-center gap-3 mb-4">
              <div class="flex items-center justify-center w-8 h-8 rounded-[var(--radius-md)] bg-[var(--color-status-warning)]/10">
                <Shield size={16} class="text-[var(--color-status-warning)]" />
              </div>
              <div>
                <h3 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
                  {$_("settings.hooks")}
                </h3>
                <p class="text-[10px] text-[var(--color-text-tertiary)]">{$_("settings.hooksDesc")}</p>
              </div>
            </div>

            <div class="space-y-2">
              {#each [
                { name: "check-pure-csharp", desc: "Blocks UnityEngine imports in Logic/Core/Systems", status: "blocker" },
                { name: "check-naming-conventions", desc: "Warns about non-PascalCase types", status: "warning" },
                { name: "check-no-linq-hotpath", desc: "Warns about LINQ on hot paths", status: "warning" },
                { name: "check-no-runtime-instantiate", desc: "Warns about runtime Instantiate/Destroy", status: "warning" },
                { name: "check-test-exists", desc: "Reminds if logic class has no test", status: "warning" },
                { name: "check-compile", desc: "Basic syntax validation", status: "warning" },
                { name: "update-progress", desc: "Logs file activity to ACTIVITY_LOG.md", status: "info" },
              ] as hook}
                <div class="flex items-center gap-3 px-3 py-2 rounded-[var(--radius-md)] bg-[var(--color-bg-elevated)]/50">
                  <div
                    class="w-2 h-2 rounded-full shrink-0"
                    class:bg-[var(--color-status-error)]={hook.status === "blocker"}
                    class:bg-[var(--color-status-warning)]={hook.status === "warning"}
                    class:bg-[var(--color-status-info)]={hook.status === "info"}
                  ></div>
                  <span class="text-[var(--text-caption)] font-mono text-[var(--color-text-primary)] w-48 shrink-0">{hook.name}</span>
                  <span class="text-[var(--text-caption)] text-[var(--color-text-secondary)] flex-1">{hook.desc}</span>
                </div>
              {/each}
            </div>
          </Panel>
        </section>
      {/if}

      <!-- Appearance Section -->
      {#if mounted}
        <section in:staggeredItem={{ index: 4, staggerDelay: 60, duration: 250, distance: 8 }}>
          <Panel class="p-5">
            <div class="flex items-center gap-3 mb-4">
              <div class="flex items-center justify-center w-8 h-8 rounded-[var(--radius-md)] bg-[var(--color-agent-reviewer)]/10">
                <Palette size={16} class="text-[var(--color-agent-reviewer)]" />
              </div>
              <div>
                <h3 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
                  {$_("settings.appearance")}
                </h3>
              </div>
            </div>

            <div class="space-y-4">
              <div>
                <div class="flex items-center justify-between mb-2">
                  <div>
                    <label class="text-[var(--text-caption)] text-[var(--color-text-secondary)]">{$_("settings.fontSize")}</label>
                    <p class="text-[10px] text-[var(--color-text-tertiary)]">{$_("settings.fontSizeDesc")}</p>
                  </div>
                  <span class="text-[var(--text-body)] font-mono text-[var(--color-accent)]">{local.fontSize}px</span>
                </div>
                <Slider
                  bind:value={local.fontSize}
                  min={10}
                  max={18}
                  step={1}
                  onchange={markDirty}
                />
              </div>

              <div class="flex items-center justify-between">
                <div>
                  <label class="text-[var(--text-caption)] text-[var(--color-text-secondary)]">{$_("settings.reducedMotion")}</label>
                  <p class="text-[10px] text-[var(--color-text-tertiary)]">{$_("settings.reducedMotionDesc")}</p>
                </div>
                <Toggle
                  bind:checked={local.reducedMotion}
                  onchange={markDirty}
                />
              </div>
            </div>
          </Panel>
        </section>
      {/if}

      <!-- Keyboard Shortcuts Section -->
      {#if mounted}
        <section in:staggeredItem={{ index: 5, staggerDelay: 60, duration: 250, distance: 8 }}>
          <Panel class="p-5">
            <div class="flex items-center gap-3 mb-4">
              <div class="flex items-center justify-center w-8 h-8 rounded-[var(--radius-md)] bg-[var(--color-agent-commit)]/10">
                <Keyboard size={16} class="text-[var(--color-agent-commit)]" />
              </div>
              <div>
                <h3 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
                  {$_("settings.keyboard")}
                </h3>
                <p class="text-[10px] text-[var(--color-text-tertiary)]">{$_("settings.keyboardDesc")}</p>
              </div>
            </div>

            <div class="space-y-2">
              {#each shortcuts as shortcut}
                <div class="flex items-center justify-between px-3 py-2 rounded-[var(--radius-md)] bg-[var(--color-bg-elevated)]/50">
                  <span class="text-[var(--text-caption)] text-[var(--color-text-primary)]">{$_(shortcut.action)}</span>
                  <div class="flex gap-1">
                    {#each shortcut.keys as key}
                      <kbd class="px-2 py-0.5 text-[10px] font-mono bg-[var(--color-bg-overlay)] text-[var(--color-text-secondary)] rounded-[var(--radius-sm)] border border-[var(--color-border-subtle)]">
                        {key === "Cmd" ? "\u2318" : key}
                      </kbd>
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </Panel>
        </section>
      {/if}

      <!-- About Section -->
      {#if mounted}
        <section in:staggeredItem={{ index: 6, staggerDelay: 60, duration: 250, distance: 8 }}>
          <Panel class="p-5">
            <div class="flex items-center gap-3 mb-3">
              <div class="flex items-center justify-center w-8 h-8 rounded-[var(--radius-md)] bg-[var(--color-bg-overlay)]">
                <Info size={16} class="text-[var(--color-text-secondary)]" />
              </div>
              <div>
                <h3 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
                  {$_("settings.about")}
                </h3>
              </div>
            </div>
            <p class="text-[var(--text-caption)] text-[var(--color-text-secondary)] mb-2">
              {$_("settings.aboutDesc")}
            </p>
            <p class="text-[var(--text-caption)] text-[var(--color-text-tertiary)]">
              {$_("settings.version")} 0.1.0
            </p>
          </Panel>
        </section>
      {/if}

    </div>
  </div>
</div>
