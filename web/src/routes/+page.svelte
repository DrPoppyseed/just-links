<script lang="ts">
  import { onMount } from "svelte";
  import { session } from "$lib/store";
  import type { Article } from "$lib/types";
  import { getSession, getArticles } from "$lib/api";
  import ArticleCard from "$lib/components/ArticleCard.svelte";
  import SignUpButton from "$lib/components/SignUpButton.svelte";

  let loading = true;
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

<div class="px-8">
  {#if $session.isLoggedIn || loading}
    {#if loading}
      <p class="mt-6">loading...</p>
    {:else}
      {#each articles as article}
        <ArticleCard {article} />
      {/each}
    {/if}
  {:else}
    <div class="py-8 flex flex-col">
      <section class="mb-12">
        <h1 class="text-4xl font-serif mb-5">
          What Octal is to Hacker News, but for Pocket.
        </h1>
        <p class="text-sm mb-5">
          Save articles with Pocket and read them whenever you like with a
          simpler interface.
        </p>
        <SignUpButton />
      </section>
      <section class="mb-12">
        <h2 class="text-2xl font-serif mb-3">Just links. Maybe a bit more.</h2>
        <img alt="preview" src="/images/preview.png" />
      </section>
      <section class="mb-8">
        <h2 class="text-2xl font-serif mb-5">And yes. It's open source.</h2>
        <p class="text-sm mb-2">
          Take a look under the hood, or contribute to the codebase from our
          GitHub repo. Maybe even give it a star if you’re feeling generous.
        </p>
        <a
          class="text-sm text-blue-600"
          href="https://github.com/DrPoppyseed/just-links"
          >DrPoppyseed/just-links</a
        >
      </section>
    </div>
  {/if}
</div>
