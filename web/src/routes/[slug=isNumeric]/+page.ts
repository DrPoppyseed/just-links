import type { PageLoad } from "../[slug=isNumeric]/$types"

export const load: PageLoad = ({ params }) => {
  return {
      slug: parseInt(params.slug)
  }
}
