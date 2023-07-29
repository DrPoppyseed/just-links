<script lang="ts">
  import { onMount } from "svelte";
  import axios from "axios";
  import { session } from "./store";

  type Article = any;
  type Articles = Option<Array<Article>>;

  let loading = false;
  let articles: Articles = null;

  // fetch session info and save in memory
  onMount(async () => {
    if ($session.isLoggedIn) {
      loading = true;

      const getArticlesRes = await axios.get<{ articles: Articles }>(
        "http://localhost:8080/articles",
        { withCredentials: true }
      );

      console.log({ getArticlesRes });

      if (getArticlesRes.data.articles) {
        articles = getArticlesRes.data.articles;
      }

      loading = false;
    }
  });
</script>

{#if $session.isLoggedIn}
  {#if loading || articles === null}
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
    <form action="http://localhost:8080/auth/authn" method="POST">
      <button type="submit">Authorize with Pocket</button>
    </form>
  </div>
{/if}
