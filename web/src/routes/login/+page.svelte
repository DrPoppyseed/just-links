<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { session } from "../store";
  import { authz } from "../api";

  let loading = false;

  onMount(async () => {
    loading = true;

    const urlParams = new URLSearchParams(window.location.search);
    const stateParam = urlParams.get("state");

    const authzRes = await authz(stateParam);

    loading = false;
    if (authzRes.status === 200 && authzRes.data.username) {
      $session.isLoggedIn = true;
      $session.username = authzRes.data.username;
      goto("/", { replaceState: true });
    } else {
      console.error(
        `failed to authorize. received response: ${JSON.stringify(authzRes)}`
      );
      return;
    }
  });
</script>

{#if loading}
  <p>loading...</p>
{/if}
