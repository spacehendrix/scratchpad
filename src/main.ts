import { mount } from "svelte";
import "./app.css";
import "./themes/tokyo-night.css";
import "./themes/catppuccin.css";
import "./themes/gruvbox.css";
import "./themes/nord.css";
import "./themes/everforest.css";
import "./themes/rose-pine.css";
import "./themes/matte-black.css";
import App from "./App.svelte";
import { settings } from "./lib/stores/settings.svelte";

// Theme applies before (and regardless of) unlock.
settings.load();

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
