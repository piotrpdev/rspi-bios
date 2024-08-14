import "tuicss/dist/tuicss.css";
import "tuicss/dist/tuicss.js";
import "./app.css";
import App from "./App.svelte";

const app = new App({
	// biome-ignore lint/style/noNonNullAssertion: we know app exists
	target: document.getElementById("app")!,
});

export default app;
