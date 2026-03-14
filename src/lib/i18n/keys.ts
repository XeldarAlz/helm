/** Type-safe i18n key constants. Keep in sync with en.json. */

export const Keys = {
  app: {
    name: "app.name",
    tagline: "app.tagline",
  },
  splash: {
    loading: "splash.loading",
    ready: "splash.ready",
  },
  nav: {
    dashboard: "nav.dashboard",
    chat: "nav.chat",
    orchestration: "nav.orchestration",
    documents: "nav.documents",
    code: "nav.code",
    git: "nav.git",
    history: "nav.history",
    settings: "nav.settings",
  },
  dashboard: {
    welcome: "dashboard.welcome",
    subtitle: "dashboard.subtitle",
    quickActions: {
      newGame: "dashboard.quickActions.newGame",
      newGameDesc: "dashboard.quickActions.newGameDesc",
      resume: "dashboard.quickActions.resume",
      resumeDesc: "dashboard.quickActions.resumeDesc",
      status: "dashboard.quickActions.status",
      statusDesc: "dashboard.quickActions.statusDesc",
    },
    pipeline: {
      idea: "dashboard.pipeline.idea",
      architecture: "dashboard.pipeline.architecture",
      planning: "dashboard.pipeline.planning",
      building: "dashboard.pipeline.building",
      complete: "dashboard.pipeline.complete",
    },
  },
  chat: {
    inputPlaceholder: "chat.inputPlaceholder",
    send: "chat.send",
    newSession: "chat.newSession",
    endSession: "chat.endSession",
  },
  orchestration: {
    title: "orchestration.title",
    phase: "orchestration.phase",
  },
  common: {
    confirm: "common.confirm",
    cancel: "common.cancel",
    close: "common.close",
    save: "common.save",
    loading: "common.loading",
    error: "common.error",
    retry: "common.retry",
  },
} as const;
