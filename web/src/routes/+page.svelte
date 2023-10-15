<script lang="ts">
  import { isLoggingIn } from "$lib/store";
  import { session } from "$lib/store";
  import LandingPage from "./LandingPage.svelte";
  import Articles from "$lib/components/Articles.svelte";
</script>

<div class="px-8 overflow-hiddenself-start">
  {#if $isLoggingIn}
    <p class="mt-6">Authenticating...</p>
  {:else if $session.isLoggedIn}
    <Articles pageNumber={0} />
  {:else}
    <LandingPage />
  {/if}
</div>
