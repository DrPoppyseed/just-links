<script lang="ts">
  import Articles from "$lib/components/Articles.svelte";

  export let data;
</script>

<div class="px-8 overflow-hiddenself-start">
  <Articles pageNumber={data.slug} />
</div>
