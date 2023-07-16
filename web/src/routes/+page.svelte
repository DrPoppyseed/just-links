<script lang="ts">
  import { onMount } from "svelte";
  import axios from "axios";

  let loading = false;

  let authUri: string | null = null;
  let accessToken: string | null = null;
  let sessionId: string | null = null;

  $: hasCredentials = !!accessToken?.length && !!sessionId?.length;

  async function getRequestToken(): Promise<void> {
    const res = await axios.post<{
      requestToken?: string | null;
      authUri?: string | null;
    }>("http://localhost:8080/auth/pocket");

    if (res.status === 200 && res.data.requestToken && res.data.authUri) {
      sessionStorage.setItem("requestToken", res.data.requestToken);
      authUri = res.data.authUri;
    } else {
      console.error(
        `failed to authenticate. received response: ${JSON.stringify(res)}`
      );
    }
  }

  onMount(async () => {
    // fetch creds from sessionStorage
    accessToken = sessionStorage.getItem("accessToken");
    sessionId = sessionStorage.getItem("sessionId");

    console.log(
      `credentials: ${accessToken}, ${sessionId}, hasCredentials:${hasCredentials}`
    );

    if (hasCredentials) {
      loading = true;
      const res = await axios.get("http://localhost:8080/articles");

      loading = false;
      console.log({ data: res.data });
    }
  });
</script>

{#if hasCredentials}
  {#if loading}
    <p>loading!</p>
  {:else}
    <p>logged in!</p>
  {/if}
{:else}
  <div class="flex flex-col items-center">
    <button
      class="border border-indigo-300 p-2"
      on:click={() => getRequestToken()}
    >
      get request token
    </button>

    {#if authUri?.length}
      <a href={authUri}>grant permission for pocket</a>
    {/if}
  </div>
{/if}
