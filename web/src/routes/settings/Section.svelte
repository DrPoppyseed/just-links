<script lang="ts">
  export let title: string;
</script>

<section class="mb-6">
  <h2 class="text-xl font-bold mb-4">{title}</h2>
  <slot />
</section>
