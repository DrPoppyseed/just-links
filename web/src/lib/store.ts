import { writable } from "svelte/store";

export type Session = {
  username: Option<string>;
  isLoggedIn: boolean;
};

export const session = writable<Session>({
  username: null,
  isLoggedIn: false,
});
