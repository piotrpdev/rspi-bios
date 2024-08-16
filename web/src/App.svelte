<script lang="ts">
import Ascii from "./lib/screens/ScanDisk.svelte";
import Startup from "./lib/screens/Startup.svelte";

// const eventSource = new EventSource('sse');

// eventSource.onmessage = (event) => {
//   console.log('Message from server ', event.data);
// }

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
