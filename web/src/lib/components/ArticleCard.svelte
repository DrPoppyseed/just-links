<script lang="ts">
  import type { Article } from "$lib/types";
  export let article: Article;

  // TODO: should probably be handled on the backend
  const formatUrl = (url: string): [string, string] => {
    const formattedUrl = new URL(url);
    return [formattedUrl.toString(), formattedUrl.hostname];
  };

  // TODO: allow user to toggle absolute or relative date added
  const dateAdded = article.timeAdded
    ? new Date(article.timeAdded * 1000).toLocaleDateString()
    : null;

  const [url, hostname] =
    article.resolvedUrl || article.givenUrl
      ? formatUrl(article.resolvedUrl || article.givenUrl || "")
      : [null, null];
</script>

<div class="border-b pt-1 pb-2 flex items-center justify-between space-x-2">
  <div>
    <!--Try to display resolved_url and fallback to given_url if null-->
    <a href={url} class="text-xs text-blue-500">{hostname}</a>
    <h3><a href={url}>{article.givenTitle}</a></h3>
    {#if article.timeAdded}
      <p class="mt-1 text-sm text-gray-500">{dateAdded}</p>
    {/if}
  </div>
  {#if article.topImageUrl}
    <img
      src={article.topImageUrl}
      alt={article.topImageUrl}
      class="w-16 h-16 object-cover"
    />
  {/if}
</div>
