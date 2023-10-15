<script lang="ts">
  import { syncArticlesService } from "$lib/syncArticlesMachine.js";
  import { syncState, rateLimits } from "$lib/store.js";
  let enableDarkMode: boolean;
  let checkboxes: ReadonlyArray<{
    id: string;
    label: string;
    subLabel: string | null;
  }> = [
    {
      id: "enableDarkMode",
      label: "Dark Mode",
      subLabel: null,
    },
    {
      id: "enableRichView",
      label: "Rich View",
      subLabel:
        "Show a picture associated with the article alongside the link and title.",
    },
    {
      id: "enableMetricsView",
      label: "Metrics View",
      subLabel:
        "Show metrics of your recent activity and amount of articles read in the app.",
    },
    {
      id: "enableAutomaticRead",
      label: 'Automatic "Read" detection',
      subLabel:
        'Set the status of the article as "read" on click. when disabled, you will have to manually set the status of the article to "read" if you wish so.',
    },
    {
      id: "enableAutomaticSync",
      label: "Automatic Sync",
      subLabel:
        "Sync your articles with pocket every time you refresh the page",
    },
  ];

  let eventMax: number;
  let eventCur: number;

  const parseEventString = (
    eventString: string,
  ): { cur: number; max: number } => {
    const regex = /data:(\d+),(\d+)\n/;
    const match = eventString.match(regex);

    if (match) {
      return {
        cur: parseInt(match[1], 10),
        max: parseInt(match[2], 10),
      };
    } else {
      throw new Error("Invalid event string format");
    }
  };

  const syncArticles = async () => {
    try {
      syncArticlesService.send("sync");

      const response = await fetch(
        `${
          import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL
        }/articles/simulated-sync`,
        {
          credentials: "include",
        },
      );

      if (response.body) {
        const reader = response.body
          .pipeThrough(new TextDecoderStream())
          .getReader();

        while (true) {
          const { value, done } = await reader.read();
          if (done) break;

          const { cur, max } = parseEventString(value);
          eventMax = max - 1;
          eventCur = cur;
        }
      }
      syncArticlesService.send("synced");
    } catch (e) {
      syncArticlesService.send("syncFailed");
      console.error(e);
    }
  };

  $: $syncArticlesService.value === "idle" &&
    (() => {
      eventMax = 0;
      eventCur = 0;
    })();

  $: syncState.update((prev) => ({
    ...prev,
    max: eventMax,
    cur: eventCur,
  }));
</script>

<div class="flex flex-col p-6">
  <section class="mb-6">
    <h2 class="text-xl font-bold mb-4">Metrics</h2>
    <div>
      <p class="opacity-50">
        <span class="font-bold">10%</span>
        (2 articles read, out of 20 articles saved)
      </p>
    </div>
  </section>
  <div class="h-px w-full border-t bg-[#E1E1E1] mb-6" />
  <section class="mb-6">
    <h2 class="text-xl font-bold mb-4">Actions</h2>
    <div>
      <div class="flex flex-col items-start mb-4">
        <button id="syncDataButton" class="font-bold" on:click={syncArticles}
          >Sync data</button
        >
        <label for="syncDataButton" class="text-xs">
          Sync all your data from Pocket with Just Links
        </label>
      </div>
      <div class="flex flex-col items-start mb-4 opacity-50">
        <button id="exportDataButton" class="font-bold" disabled
          >Export data</button
        >
        <label for="exportDataButton" class="text-xs">
          Download all the links you have saved
        </label>
      </div>
      <div class="flex flex-col items-start opacity-50">
        <button id="signOutButton" class="font-bold" disabled>Sign out</button>
        <label for="signOutButton" class="text-xs">
          peaske16180@gmail.com
        </label>
      </div>
    </div>
  </section>
  <div class="h-px w-full border-t bg-[#E1E1E1] mb-6" />
  <section class="mb-6">
    <h2 class="text-xl font-bold mb-4">Settings</h2>
    <div class="opacity-50 grid grid-cols-[auto,40px] gap-x-16 gap-y-4">
      {#each checkboxes as checkbox (checkbox.id)}
        <div class="flex flex-col self-start">
          <label for={checkbox.id}>{checkbox.label}</label>
          {#if checkbox.subLabel != null}
            <label for={checkbox.id} class="text-xs">
              {checkbox.subLabel}
            </label>
          {/if}
        </div>
        <input
          id={checkbox.id}
          type="checkbox"
          class="justify-self-end mr-2"
          bind:checked={enableDarkMode}
        />
      {/each}
    </div>
  </section>
  <div class="h-px w-full border-t bg-[#E1E1E1] mb-6" />
  <section class="mb-6">
    <h2 class="text-xl font-bold mb-4">API Usage</h2>
    <div class="opacity-50 grid grid-cols-[auto,60px] gap-x-16 gap-y-4">
      <div class="flex flex-col self-start">
        <p>User Limit</p>
        <p class="text-xs">Your current Pocket API rate limit.</p>
      </div>
      <div class="justify-self-end mr-2">
        <p>{$rateLimits.userLimit || "--"}</p>
      </div>
      <div class="flex flex-col self-start">
        <p>User Remaining</p>
        <p class="text-xs">
          Number of calls remaining before hitting your Pocket API rate limit.
        </p>
      </div>
      <div class="justify-self-end mr-2">
        <p>{$rateLimits.userRemaining || "--"}</p>
      </div>
      <div class="flex flex-col self-start">
        <p>User Reset</p>
        <p class="text-xs">Seconds until your Pocket API rate limit resets.</p>
      </div>
      <div class="justify-self-end mr-2">
        <p>{$rateLimits.userReset || "--"}</p>
      </div>
    </div>
  </section>
</div>

<style>
  input[type="checkbox"] {
    appearance: none;
    background-color: #fff;
    width: 20px;
    height: 20px;
    border: 1px solid #888888;
    border-radius: 4px;
    display: grid;
    place-content: center;
  }

  input[type="checkbox"]::before {
    content: "";
    width: 0.9em;
    height: 0.9em;
    border-radius: 2px;
    transform: scale(0);
    box-shadow: inset 1em 1em #888888;
  }

  input[type="checkbox"]:checked::before {
    transform: scale(1);
  }
</style>
