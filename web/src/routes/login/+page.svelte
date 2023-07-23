<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import axios from "axios";
    import { session } from "../store";

  let token: string | null = null;
  let loading = false;

  onMount(async () => {
    loading = true;

    const res = await axios.post<{
      username: Option<string>;
    }>("http://localhost:8080/auth/authz");

    loading = false;
    if (res.status === 200 && res.data.username) {
      $session.isLoggedIn = true;
      $session.username = res.data.username
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
