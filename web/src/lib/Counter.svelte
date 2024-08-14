<script lang="ts">
let count = 0;
const increment = () => {
	count += 1;
};
</script>

<button on:click={increment}>
  <p>count</p> is {count}
</button>

<style lang="scss">
	button {
    color: purple;
    font-family: 'Comic Sans MS', cursive;
    font-size: 2em;
    
    p {
      color: red;
    }
	}
</style>
