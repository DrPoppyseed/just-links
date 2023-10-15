import { writable } from "svelte/store";
import type { RateLimits } from "./types";
import { z } from "zod";

export type Session = {
  username: Option<string>;
  isLoggedIn: boolean;
};

export const session = writable<Session>({
  username: null,
  isLoggedIn: false,
});

export const isLoggingIn = writable<boolean>(false);

export const rateLimits = writable<RateLimits>({
  userLimit: null,
  userRemaining: null,
  userReset: null,
})

export const syncStateSchema = z.object({
  max: z.number(),
  min: z.number().optional().default(0),
  cur: z.number().optional().default(0),
})
export type SyncState = z.infer<typeof syncStateSchema>;
export const syncState = writable<SyncState>({
  max: 0,
  min: 0,
  cur: 0,
});