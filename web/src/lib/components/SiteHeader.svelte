<script lang="ts">
  import Options from "$lib/icons/Options.svelte";
  import { syncState, session, isLoggingIn } from "$lib/store";
  import { syncArticlesService } from "$lib/syncArticlesMachine";
</script>

<header class="top-0 w-full border-b relative">
  <div class="container flex h-14 p-6 items-center justify-between">
    <div class="mr-4">
      <a href="/">
        <span class="font-bold">
          {$syncArticlesService.context.title}
        </span>
      </a>
    </div>

    {#if $session.isLoggedIn}
      <a href="/options">
        <Options />
      </a>
    {:else if !$isLoggingIn}
      <form
        action={`${import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL}/auth/authn`}
        method="POST"
      >
        <button type="submit" class="text-sm">Get started</button>
      </form>
    {:else}
      <div />
    {/if}
  </div>
  {#if $syncState.max > 0}
    <progress
      value={$syncState.cur}
      max={$syncState.max}
      class="h-1 w-full absolute bottom-0"
    />
  {/if}
</header>

<style>
  progress::-webkit-progress-bar {
    background-color: transparent;
  }
  progress::-webkit-progress-value {
    @apply bg-green-600;
  }
</style>
