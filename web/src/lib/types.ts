import { z } from "zod";

export const itemImageSchema = z.object({
  itemId: z.string(),
  imageId: z.string(),
  src: z.string(),
  width: z.string(),
  height: z.string(),
  caption: z.string(),
  credit: z.string(),
});
export type ItemImage = z.infer<typeof itemImageSchema>;

export const itemVideoSchema = z.object({
  itemId: z.string(),
  imageId: z.string(),
  src: z.string(),
  width: z.string(),
  length: z.string().optional(),
  vid: z.string(),
});
export type ItemVideo = z.infer<typeof itemVideoSchema>;

export const articleSchema = z.object({
  itemId: z.string(),
  resolveId: z.string(),
  givenUrl: z.string(),
  givenTitle: z.string(),
  resolvedUrl: z.string(),
  resolvedTitle: z.string(),
  favorite: z.string(),
  status: z.string(),
  timeAdded: z.string().optional(),
  timeUpdated: z.string().optional(),
  timeRead: z.string().optional(),
  timeFavorited: z.string().optional(),
  sortId: z.number().optional(),
  excerpt: z.string(),
  isArticle: z.string(),
  isIndex: z.string(),
  hasImage: z.string(),
  hasVideo: z.string(),
  wordCount: z.string(),
  tags: z.string().optional(),
  authors: z.string().optional(),
  images: itemImageSchema.array().optional(),
  videos: itemVideoSchema.array().optional(),
  lang: z.string().optional(),
  timeToRead: z.number().optional(),
  listenDurationEstimate: z.number().optional(),
  topImageUrl: z.string().optional(),
  domainMetadata: z.any().optional(),
});
export type Article = z.infer<typeof articleSchema>;

export const apiGetSessionResSchema = z.object({
  username: z.string().optional(),
});
export type ApiGetSessionRes = z.infer<typeof apiGetSessionResSchema>;

export const apiGetArticlesResSchema = z.object({
  articles: articleSchema.array().default([]),
});
export type ApiGetArticlesRes = z.infer<typeof apiGetArticlesResSchema>;

export const apiAuthzResSchema = z.object({
  username: z.string().optional(),
});
export type ApiAuthzRes = z.infer<typeof apiAuthzResSchema>;
