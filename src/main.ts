import "./app.css";
import { initI18n } from "$lib/i18n";
import App from "./App.svelte";
import { mount } from "svelte";

initI18n();

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
