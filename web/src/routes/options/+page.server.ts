import { handleLoginRedirect } from "$lib/utils";
import { redirect } from "@sveltejs/kit";
import type { PageServerLoad } from "../[slug=isNumeric]/$types";

export const load: PageServerLoad = async (event) => {
	const session = await event.locals.getSession();
	if (!session) {
		throw redirect(302, handleLoginRedirect(event));
	}
}