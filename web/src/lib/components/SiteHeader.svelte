<script lang="ts">
  import { session } from "$lib/store";
  import Options from "$lib/icons/Options.svelte";
</script>

<header class="top-0 w-full border-b">
  <div class="container flex h-14 p-6 items-center justify-between">
    <div class="mr-4">
      <a href="/">
        <span class="font-bold">Just Links</span>
      </a>
    </div>

    {#if $session.isLoggedIn}
      <Options />
    {:else}
      <form
        action={`${import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL}/auth/authn`}
        method="POST"
      >
        <button type="submit" class="text-sm">Get started</button>
      </form>
    {/if}
  </div>
</header>
