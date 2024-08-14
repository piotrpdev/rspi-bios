<script lang="ts">
import Ascii from "./lib/screens/ScanDisk.svelte";
import Startup from "./lib/screens/Startup.svelte";

// const socket = new WebSocket("ws://localhost:3000/ws");
// socket.addEventListener("message", (event) => {
// 	console.log("Message from server ", event.data);
// 	// const code = document.createElement("code")
// 	// code.innerText = event.data;
// 	// document.querySelector("#messages")?.appendChild(code);
// });

// biome-ignore lint/style/useConst: binded var
let isStartupFinished = false;
</script>

<main>
  {#if !isStartupFinished}
    <Startup bind:isFinished={isStartupFinished} />
  {:else}
    <Ascii />
  {/if}
</main>

<style>
</style>
