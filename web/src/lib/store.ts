import { writable } from "svelte/store";
import type { RateLimits } from "./types";

export type Session = {
  username: Option<string>;
  isLoggedIn: boolean;
};

export const session = writable<Session>({
  username: null,
  isLoggedIn: false,
});

export const rateLimits = writable<RateLimits>({
  userLimit: null,
  userRemaining: null,
  userReset: null,
})