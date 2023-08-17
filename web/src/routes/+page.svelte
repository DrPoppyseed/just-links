<script lang="ts">
  import { onMount } from "svelte";
  import axios from "axios";
  import { session } from "./store";
  import {type Article} from './schemas'

  let loading = false;
  let articles: Option<Array<Article>> = null;

  // fetch session info and save in memory
  onMount(async () => {
    if ($session.isLoggedIn) {
      loading = true;

      const getArticlesRes = await axios.get<{ articles: Array<Article> }>(
        "http://localhost:8080/articles",
        { withCredentials: true }
      );

      if (getArticlesRes.data.articles) {
        articles = getArticlesRes.data.articles;
      }

      loading = false;
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
    <form action="http://localhost:8080/auth/authn" method="POST">
      <button type="submit">Authorize with Pocket</button>
    </form>
  </div>
{/if}


