export interface Feed {
  id: string;
  title: string;
  url: string;
  feedType: string | null;
  siteUrl: string | null;
  description: string | null;
  iconUrl: string | null;
  createdAt: string;
  updatedAt: string;
  lastFetchedAt: string | null;
  etag: string | null;
  lastModified: string | null;
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
  categories: string | null;
}
