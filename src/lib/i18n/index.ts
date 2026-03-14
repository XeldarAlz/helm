import { init, register, getLocaleFromNavigator, locale, t, _ } from "svelte-i18n";

register("en", () => import("./en.json"));

export function initI18n() {
  init({
    fallbackLocale: "en",
    initialLocale: getLocaleFromNavigator() ?? "en",
  });
}

export { locale, t, _ };
