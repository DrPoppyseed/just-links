<script lang="ts">
  export let pageNumber: number;
</script>

<nav class="my-4">
  <ul class="flex items-center justify-between">
    {#if pageNumber > 0}
      <li><a href={`/${pageNumber - 1}`}>Previous</a></li>
    {:else}
      <div />
    {/if}
    <li><a href={`/${pageNumber + 1}`}>Next</a></li>
  </ul>
</nav>
