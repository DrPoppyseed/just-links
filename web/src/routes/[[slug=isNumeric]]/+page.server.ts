import type { PageServerLoad } from "./$types";
import type {
  ApiGetArticlesRes,
  Article,
  RateLimits,
  Session,
} from "$lib/types";

export const load: PageServerLoad = async (
  event,
): Promise<{
  session: Session | null;
  articles: Array<Article>;
  rateLimits: RateLimits | null;
  pageNumber: number;
}> => {
  const { session } = await event.parent();
  const pageNumber = parseInt(event.params.slug || "0");
  const ID = event.cookies.get("ID");

  if (!session?.username) {
    return {
      session: null,
      articles: [],
      rateLimits: null,
      pageNumber,
    };
  }

  try {
    const {
      data: { articles },
      rateLimits,
    } = await fetch(
      `${
        import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL
      }/articles?page=${pageNumber}`,
      {
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json",
          Cookie: `ID=${ID}; SameSite=None; Secure; HttpOnly; Path=/;`,
        },
        credentials: "include",
      },
    )
      .then((res) => {
        if (!res.ok) {
          throw new Error(res.statusText);
        }

        if (res.headers.get("cache-control")) {
          event.setHeaders({
            "cache-control": res.headers.get("cache-control")!,
          });
        }

        return res.json() as Promise<ApiGetArticlesRes>;
      })
      .then((res): ApiGetArticlesRes => {
        const articles = res.data.articles.sort((a, b) => {
          return (a.timeAdded || -1) > (b.timeAdded || -1) ? -1 : 1;
        });

        return {
          ...res,
          data: { articles },
        };
      });

    return {
      session,
      articles,
      rateLimits,
      pageNumber,
    };
  } catch (e) {
    console.error(e);
    return {
      session,
      articles: [],
      rateLimits: null,
      pageNumber,
    };
  }
};
