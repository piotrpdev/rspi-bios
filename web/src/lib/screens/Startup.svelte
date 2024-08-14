<script lang="ts">
import Boot from "./pages/Boot.svelte";
import SpecTable from "./pages/SpecTable.svelte";

export let isFinished = false;

// biome-ignore lint/style/useConst: binded var
let isBootFinished = false;
// biome-ignore lint/style/useConst: binded var
let isSpecTableFinished = false;

$: isFinished = isSpecTableFinished;
</script>

<div id="startup" class="tui-screen-1024-768 black white-text" style="padding: 2px">
    {#if !isBootFinished}
      <Boot bind:isFinished={isBootFinished} />
    {:else if !isSpecTableFinished}
      <SpecTable bind:isFinished={isSpecTableFinished} />
    {/if}
</div>
