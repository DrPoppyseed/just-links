<script lang="ts">
  import type { Article } from "$lib/types";
  export let article: Article;

  // TODO: should probably be handled on the backend
  const formatUrl = (): [string, string] => {
    const url = new URL(article.resolvedUrl || article.givenUrl);
    return [url.toString(), url.hostname];
  };

  // TODO: allow user to toggle absolute or relative date added
  const dateAdded = article.timeAdded
    ? new Date(article.timeAdded).toLocaleDateString()
    : null;
  const [url, hostname] = formatUrl();
</script>

<div class="border-b pt-1 pb-2 flex items-center space-x-2">
  <div>
    <!--Try to display resolved_url and fallback to given_url if null-->
    <a href={url} class="text-xs text-blue-500">{hostname}</a>
    <h3>{article.givenTitle}</h3>
    {#if article.timeAdded}
      <p class="text-sm text-gray-500">{dateAdded}</p>
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
