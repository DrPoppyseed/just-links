<script lang="ts">
 import "../app.css";
  import SiteHeader from "$lib/components/SiteHeader.svelte";
  import SiteFooter from "$lib/components/SiteFooter.svelte";

  let networkStatus: boolean = true;

  window.addEventListener("load", () => {
    networkStatus = navigator.onLine;

    const handleNetworkChange = () => {
      networkStatus = navigator.onLine;
    };

    window.addEventListener("online", handleNetworkChange);
    window.addEventListener("offline", handleNetworkChange);
  });
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
  <SiteFooter />
  {#if !networkStatus}
    <p class="bottom-0">offline!</p>
  {/if}
</div>
