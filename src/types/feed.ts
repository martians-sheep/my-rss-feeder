export interface Feed {
  id: string;
  title: string;
  url: string;
  siteUrl: string | null;
  description: string | null;
  iconUrl: string | null;
  createdAt: string;
  updatedAt: string;
  lastFetchedAt: string | null;
  etag: string | null;
  lastModified: string | null;
}

export type DatePreset = "today" | "week" | "month" | "all" | "custom";

export interface DateFilter {
  preset: DatePreset;
  customDate: string | null; // "YYYY-MM-DD"
}

export interface Article {
  id: string;
  feedId: string;
  entryId: string;
  title: string;
  url: string | null;
  summary: string | null;
  content: string | null;
  author: string | null;
  publishedAt: string | null;
  isRead: boolean;
  readAt: string | null;
  ogImageUrl: string | null;
  ogImageLocal: string | null;
  ogDescription: string | null;
  ogFetched: boolean;
  createdAt: string;
  feedTitle: string | null;
}
