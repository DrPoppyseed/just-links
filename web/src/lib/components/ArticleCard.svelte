<script lang="ts">
  import type { Article } from "$lib/types";
  export let article: Article;
  export let articleNumber: number;

  let imageUrl = article.topImageUrl;

  const fallbackImage = (url: string) => {
    imageUrl = `https://s2.googleusercontent.com/s2/favicons?domain_url=${url}`;
  };

  // TODO: should probably be handled on the backend
  const formatUrl = (url: string): [string, string] => {
    const formattedUrl = new URL(url);
    return [formattedUrl.toString(), formattedUrl.hostname];
  };

  // TODO: allow user to toggle absolute or relative date added
  const dateAdded = article.timeAdded
    ? new Date(article.timeAdded * 1000).toLocaleDateString()
    : null;

  const [url, hostname] = formatUrl(
    article.resolvedUrl || article.givenUrl || "",
  );
</script>

<article class="w-full py-2 grid grid-cols-[auto,64px] gap-x-2">
  <div class="self-start">
    <!--Try to display resolved_url and fallback to given_url if null-->

    <p class="text-sm text-gray-500">
      {articleNumber}.
      <a href={url} class="text-sm text-blue-500 break-words">{hostname}</a>
    </p>
    <a href={url} class="break-words">{article.givenTitle || hostname}</a>
    {#if article.timeAdded}
      <p class="mt-1 text-sm text-gray-500">{dateAdded}</p>
    {/if}
  </div>
  {#if imageUrl}
    <img
      src={imageUrl}
      alt={imageUrl}
      on:error={() => fallbackImage(url)}
      class="self-start w-16 h-16 object-cover"
    />
  {/if}
</article>
