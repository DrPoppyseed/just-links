<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import axios from "axios";

  let token: string | null = null;
  let loading = false;

  onMount(async () => {
    loading = true;
    const requestToken = sessionStorage.getItem("requestToken");

    if (requestToken?.length) {
      token = requestToken;
    } else {
      console.error("requestToken missing from sessionStorage");
      goto("/");
    }

    const res = await axios.post<{
      accessToken?: string | null;
      sessionId?: string | null;
    }>(
      "http://localhost:8080/auth/authorize",
      {
        requestToken,
      },
      {
        headers: {
          "Content-Type": "application/json",
        },
      }
    );

    loading = false;
    if (res.status === 200 && res.data.accessToken && res.data.sessionId) {
      sessionStorage.setItem("accessToken", res.data.accessToken);
      sessionStorage.setItem("sessionId", res.data.sessionId);
      goto("/", { replaceState: true });
    } else {
      console.error(
        `failed to authorize. received response: ${JSON.stringify(res)}`
      );
      return;
    }
  });
</script>

{#if loading}
  <p>loading...</p>
{/if}
