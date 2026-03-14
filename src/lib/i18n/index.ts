import { init, register, getLocaleFromNavigator, locale, t, _, waitLocale as _waitLocale } from "svelte-i18n";

register("en", () => import("./en.json"));

export function initI18n() {
  init({
    fallbackLocale: "en",
    initialLocale: getLocaleFromNavigator() ?? "en",
  });
}

export const waitLocale = _waitLocale;
export { locale, t, _ };
