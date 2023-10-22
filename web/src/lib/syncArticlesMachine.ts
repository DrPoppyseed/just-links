import { assign, createMachine, interpret } from "xstate";

const syncArticlesMachine = createMachine({
  id: "syncArticles",
  initial: "idle",
  context: {
    title: "Just Links",
  },
  states: {
    idle: {
      entry: assign({ title: "Just Links" }),
      on: {
        sync: {
          target: "syncing",
        },
      },
    },
    syncing: {
      entry: assign({
        title: "Syncing...",
      }),
      on: {
        synced: {
          target: "synced",
        },
        syncFailed: {
          target: "idle",
        },
      },
    },
    synced: {
      entry: assign({ title: "Synced!" }),
      after: {
        2000: {
          target: "idle",
        },
      },
    },
  },
  predictableActionArguments: true,
});

export const syncArticlesService = interpret(syncArticlesMachine).start();
