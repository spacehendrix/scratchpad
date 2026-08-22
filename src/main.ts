import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { settings } from "./lib/stores/settings.svelte";

// Every theme stylesheet, bundled eagerly — selection is just data-theme.
import.meta.glob("./themes/*.css", { eager: true });

// Theme applies before (and regardless of) unlock.
settings.load();

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
