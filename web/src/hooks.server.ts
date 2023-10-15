import { getSession } from "$lib/api";
import type { Handle } from "@sveltejs/kit";

export const handle: Handle = async ({ event, resolve }) => {
	event.locals.getSession = async () => await getSession().then(res => res.data);
 
	return await resolve(event)
};