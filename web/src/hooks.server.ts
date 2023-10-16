import type { Session } from "$lib/types";
import type { Handle } from "@sveltejs/kit";

export const handle: Handle = async ({ event, resolve }) => {
  event.locals.getSession = async () =>
    await fetch(
      `${import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL}/auth/session`,
			{
				credentials: 'include'
			}
    )
      .then((res) => {
				if (!res.ok) {
					throw new Error('Get session failed')
				}
				return res.json() as Promise<Session>
			})

  return await resolve(event);
};
