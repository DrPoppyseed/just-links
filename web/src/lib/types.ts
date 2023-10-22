import { z } from "zod";

export const rateLimitsSchema = z.object({
  userLimit: z.number().optional().nullable(),
  userRemaining: z.number().optional().nullable(),
  userReset: z.number().optional().nullable(),
});
export type RateLimits = z.infer<typeof rateLimitsSchema>;

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
  resolveId: z.string().optional(),
  givenUrl: z.string().optional(),
  givenTitle: z.string().optional(),
  resolvedUrl: z.string().optional(),
  resolvedTitle: z.string().optional(),
  favorite: z.string().optional(),
  status: z.string(),
  timeAdded: z.number().optional(),
  timeUpdated: z.number().optional(),
  timeRead: z.number().optional(),
  timeFavorited: z.number().optional(),
  sortId: z.number().optional(),
  excerpt: z.string().optional(),
  isArticle: z.string().optional(),
  isIndex: z.string().optional(),
  hasImage: z.string().optional(),
  hasVideo: z.string().optional(),
  wordCount: z.string().optional(),
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
export type Session = z.infer<typeof apiGetSessionResSchema>;

export const apiGetArticlesResSchema = z.object({
  data: z.object({ articles: articleSchema.array().default([]) }),
  rateLimits: rateLimitsSchema,
});
export type ApiGetArticlesRes = z.infer<typeof apiGetArticlesResSchema>;

export const apiAuthzResSchema = z.object({
  username: z.string().optional(),
});
export type ApiAuthzRes = z.infer<typeof apiAuthzResSchema>;

export type CssClasses = string;
