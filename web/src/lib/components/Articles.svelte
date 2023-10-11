<script lang="ts">
  import { PER_PAGE, getArticles } from "$lib/api";
  import { rateLimits } from "$lib/store";
  import type { Article } from "$lib/types";
  import ArticleCard from "./ArticleCard.svelte";
  import Pagination from "./Pagination.svelte";

  export let pageNumber: number;
  let articles: Array<Article> = [];
  let status: "loading" | "success" | "error" = "loading";

  $: new Promise((resolve) => {
    status = "loading";
    resolve(true);
  })
    .then(() => getArticles(pageNumber))
    .then((res) => {
      if (res.status == 200 && res.data) {
        status = "success";
        articles = res.data.data.articles;
        $rateLimits = res.data.rateLimits;
      } else {
        status = "error";
      }
    })
    .catch((e) => {
      status = "error";
    });
</script>

{#if status === "loading"}
  <p class="mt-6">loading...</p>
{:else if status === "success"}
  {#each articles as article, i}
    <ArticleCard {article} articleNumber={pageNumber * PER_PAGE + i + 1} />
    <div class="border-b h-px w-full bg-[#E1E1E1]" />
  {/each}
  <Pagination bind:pageNumber />
{:else}
  <p class="mt-6">Failed to fetch articles</p>
{/if}
