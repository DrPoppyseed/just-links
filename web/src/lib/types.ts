import { z } from "zod";

export const itemImageSchema = z.object({
  item_id: z.string(),
  image_id: z.string(),
  src: z.string(),
  width: z.string(),
  height: z.string(),
  caption: z.string(),
  credit: z.string(),
})
export type ItemImage = z.infer<typeof itemImageSchema>

export const itemVideoSchema = z.object({
  item_id: z.string(),
  image_id: z.string(),
  src: z.string(),
  width: z.string(),
  length: z.string().optional(),
  vid: z.string()
})
export type ItemVideo = z.infer<typeof itemVideoSchema>

export const articleSchema = z.object({
  item_id: z.string(),
  resolve_id: z.string(),
  given_url: z.string(),
  given_title: z.string(),
  resolved_url: z.string(),
  resolved_title: z.string(),
  favorite: z.string(),
  status: z.string(),
  time_added: z.string().optional(),
  time_updated: z.string().optional(),
  time_read: z.string().optional(),
  time_favorited: z.string().optional(),
  sort_id: z.number().optional(),
  excerpt: z.string(),
  is_article: z.string(),
  is_index: z.string(),
  has_image: z.string(),
  has_video: z.string(),
  word_count: z.string(),
  tags: z.string().optional(),
  authors: z.string().optional(),
  images: itemImageSchema.array().optional(),
  videos: itemVideoSchema.array().optional(),
  lang: z.string().optional(),
  time_to_read: z.number().optional(),
  listen_duration_estimate: z.number().optional(),
  top_image_url: z.string().optional(),
  domain_metadata: z.any().optional()
})
export type Article = z.infer<typeof articleSchema>

export const apiGetSessionResSchema = z.object({
  username: z.string().optional()
})
export type ApiGetSessionRes = z.infer<typeof apiGetSessionResSchema>

export const apiGetArticlesResSchema = z.object({
  articles: articleSchema.array().default([])
})
export type ApiGetArticlesRes = z.infer<typeof apiGetArticlesResSchema>

export const apiAuthzResSchema = z.object({
  username: z.string().optional()
})
export type ApiAuthzRes = z.infer<typeof apiAuthzResSchema>
