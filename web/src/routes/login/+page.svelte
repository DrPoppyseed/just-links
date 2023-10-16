<script lang="ts">
  import { goto } from "$app/navigation";
  import type { ApiAuthzRes } from "$lib/types";
  import { onMount } from "svelte";

  type FetchStatus = "success" | "loading" | "error";
  let status: FetchStatus = "loading";

  onMount(async () => {
    status = "loading";

    const urlParams = new URLSearchParams(window.location.search);
    const stateParam = urlParams.get("state");

    try {
      if (!stateParam) {
        status = "error";
        return;
      }

      const authzRes = await fetch(
        `${import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL}/auth/authz`,
        {
          method: "POST",
          body: JSON.stringify({ state: stateParam }),
          headers: {
            "Content-Type": "application/json",
            Accept: "application/json",
          },
          credentials: "include",
        },
      ).then((res) => {
        if (!res.ok) {
          throw new Error("Failed to authorize.");
        }

        return res.json() as Promise<ApiAuthzRes>;
      });

      if (authzRes.username) {
        status = "success";
      } else {
        status = "error";
      }
      goto("/", { replaceState: true });
    } catch (error) {
      status = "error";
      console.error(`Failed to authorize!`);
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
