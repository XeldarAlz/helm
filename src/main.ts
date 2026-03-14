import "./app.css";
import { initI18n, waitLocale } from "$lib/i18n";
import App from "./App.svelte";
import { mount } from "svelte";

initI18n();

waitLocale().then(() => {
  mount(App, {
    target: document.getElementById("app")!,
  });
});
