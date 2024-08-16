<script lang="ts">
import Boot from "@lib/pages/Boot.svelte";
import SpecTable from "@lib/pages/SpecTable.svelte";

// biome-ignore lint/style/useConst: binded var
export let isFinished = false;

// biome-ignore lint/style/useConst: binded var
let isBootFinished = false;
</script>

<div id="startup" class="tui-screen-1024-768 black white-text" style="padding: 2px">
    {#if !isBootFinished}
      <Boot bind:isFinished={isBootFinished} />
    {:else}
      <SpecTable bind:isFinished={isFinished} />
    {/if}
</div>
