import type { Session } from "$lib/types";
import type { Handle } from "@sveltejs/kit";

export const handle: Handle = async ({ event, resolve }) => {
  const ID = event.cookies.get("ID");

  if (!ID) {
    return await resolve(event);
  }

  const session = await fetch(
    `${import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL}/auth/session`,
    {
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        Cookie: `ID=${ID}`,
      },
      credentials: "include",
    },
  ).then((res) => {
    if (!res.ok) {
      throw new Error("Get session failed");
    }
    return res.json() as Promise<Session>;
  });

  if (session) {
    event.locals.session = {
      username: session.username,
    };
  }

  return await resolve(event);
};
