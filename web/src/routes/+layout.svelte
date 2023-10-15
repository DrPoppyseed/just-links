<script lang="ts">
  import "../app.css";
  import SiteHeader from "$lib/components/SiteHeader.svelte";
  import { getSession } from "$lib/api";
  import { isLoggingIn, session } from "$lib/store";
  import { onMount } from "svelte";

  let networkStatus: boolean = true;

  window.addEventListener("load", () => {
    networkStatus = navigator.onLine;

    const handleNetworkChange = () => {
      networkStatus = navigator.onLine;
    };

    window.addEventListener("online", handleNetworkChange);
    window.addEventListener("offline", handleNetworkChange);
  });

  onMount(async () => {
    try {
      $isLoggingIn = true;
      const getSessionRes = await getSession();
      if (getSessionRes.status == 200 && getSessionRes.data.username) {
        $session.isLoggedIn = true;
        $session.username = getSessionRes.data.username;
      } else {
        $session.isLoggedIn = false;
        $session.username = null;
      }
    } catch (e) {
      console.error();
    } finally {
      $isLoggingIn = false;
    }
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
  {#if !networkStatus}
    <p class="bottom-0">offline!</p>
  {/if}
</div>
