<script lang="ts">
  import { rateLimits } from "$lib/store";
  import LandingPage from "./LandingPage.svelte";
  import ArticleCard from "./ArticleCard.svelte";
  import Pagination from "./Pagination.svelte";
  import { ARTICLES_PER_PAGE } from "$lib/utils";

  export let data;

  if (data.session?.username) {
    $rateLimits.userLimit = data.rateLimits?.userLimit;
    $rateLimits.userRemaining = data.rateLimits?.userRemaining;
    $rateLimits.userReset = data.rateLimits?.userReset;
  }
</script>

<div class="px-8 overflow-hiddenself-start">
  {#if data.session?.username}
    {#each data.articles as article, i}
      <ArticleCard
        {article}
        articleNumber={data.pageNumber * ARTICLES_PER_PAGE + i + 1}
      />
      <div class="border-b h-px w-full bg-[#E1E1E1]" />
    {/each}
    <Pagination pageNumber={data.pageNumber} />
  {:else}
    <LandingPage />
  {/if}
</div>
