import { getSession } from "../lib/api";
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async () => {
	return getSession().then(res => res.data);
};