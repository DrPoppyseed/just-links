import type { ParamMatcher } from "@sveltejs/kit"

export const match: ParamMatcher = value => {
  return /^[0-9]{1,4}$/.test(value)
}