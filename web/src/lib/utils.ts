import type { RequestEvent } from "@sveltejs/kit";
import type { ClassValue } from "clsx";
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export const ARTICLES_PER_PAGE = 30;

export const cn = (...inputs: Array<ClassValue>) => twMerge(clsx(inputs));

export const handleLoginRedirect = (event: RequestEvent) => {
  const redirectTo = event.url.pathname + event.url.search;
  return `/login?redirectTo=${redirectTo}`;
};
