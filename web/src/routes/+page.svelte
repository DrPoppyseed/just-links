<script lang="ts">
  import { onMount } from "svelte";
  import { session } from "./store";
  import type { Article } from "./schemas";
  import { getSession, getArticles } from "./api";

  let loading = false;
  let articles: Array<Article> = [];

  // fetch session info and save in memory
  onMount(async () => {
    loading = true;

    // the service-worker should have a cached response for this, so we don't
    // expect to actually fetch from the backend for most cases
    const getSessionRes = await getSession();
    if (getSessionRes.status == 200 && getSessionRes.data.username) {
      $session.isLoggedIn = true;
      $session.username = getSessionRes.data.username;

      const getArticlesRes = await getArticles();
      if (getArticlesRes.data.articles) {
        articles = getArticlesRes.data.articles;
      }

      loading = false;
    } else {
      loading = false;
      $session.isLoggedIn = false;
      $session.username = null;
    }
  });
</script>

{#if $session.isLoggedIn}
  {#if loading === null}
    <p>loading!</p>
  {:else}
    <ul>
      {#each articles as article, i}
        <li>{i} - {JSON.stringify(article)}</li>
      {/each}
    </ul>
  {/if}
{:else}
  <div class="flex flex-col items-center">
    <form
      action={`${import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL}/auth/authn`}
      method="POST"
    >
      <button type="submit">Authorize with Pocket</button>
    </form>
  </div>
{/if}
