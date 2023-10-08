<script lang="ts">
  import { onMount } from "svelte";
  import { session } from "$lib/store";
  import { getSession } from "$lib/api";
  import LandingPage from "./LandingPage.svelte";
  import Articles from "$lib/components/Articles.svelte";

  let loading = true;

  // fetch session info and save in memory
  onMount(async () => {
    loading = true;

    // the service-worker should have a cached response for this, so we don't
    // expect to actually fetch from the backend for most cases
    const getSessionRes = await getSession();
    if (getSessionRes.status == 200 && getSessionRes.data.username) {
      $session.isLoggedIn = true;
      $session.username = getSessionRes.data.username;
      loading = false;
    } else {
      loading = false;
      $session.isLoggedIn = false;
      $session.username = null;
    }
  });
</script>

<div class="px-8 overflow-hiddenself-start">
  {#if loading}
    <p class="mt-6">Authenticating...</p>
  {:else if $session.isLoggedIn}
    <Articles pageNumber={0} />
  {:else}
    <LandingPage />
  {/if}
</div>
