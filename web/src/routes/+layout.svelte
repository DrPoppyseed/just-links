<script lang="ts">
  import "../app.css";
  import SiteHeader from "$lib/components/SiteHeader.svelte";
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" />
  <link
    href="https://fonts.googleapis.com/css2?family=Cormorant+Garamond:wght@500&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<div class="relative flex min-h-screen flex-col">
  <SiteHeader />
  <div class="flex-1">
    <slot />
  </div>
</div>
