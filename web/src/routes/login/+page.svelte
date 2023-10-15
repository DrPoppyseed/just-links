<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { session } from "$lib/store";
  import { authz } from "$lib/api";

  type FetchStatus = "success" | "loading" | "error";
  let status: FetchStatus = "loading";

  onMount(async () => {
    status = "loading";

    const urlParams = new URLSearchParams(window.location.search);
    const stateParam = urlParams.get("state");

    try {
      const authzRes = await authz(stateParam);

      if (authzRes.status === 200 && authzRes.data.username) {
        $session.isLoggedIn = true;
        $session.username = authzRes.data.username;
        status = "success";
      } else {
        status = "error";
      }
      goto("/", { replaceState: true });
    } catch (error) {
      status = "error";
      console.error(
        `failed to authorize. received response: ${JSON.stringify(error)}`,
      );
      goto("/login", { replaceState: true });
    }
  });
</script>

<div class="flex flex-col p-6">
  {#if status == "loading"}
    <p>loading...</p>
  {:else if status == "error"}
    <p>
      Failed to authenticate. Please
      <a href="/" class="text-blue-600"> try again. </a>
    </p>
  {/if}
</div>
