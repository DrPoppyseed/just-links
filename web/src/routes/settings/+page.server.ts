import { handleLoginRedirect } from "$lib/utils";
import { redirect } from "@sveltejs/kit";
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = async (event) => {
	const { session } = await event.parent()
	if (!session?.username) {
		throw redirect(302, handleLoginRedirect(event));
	}
}