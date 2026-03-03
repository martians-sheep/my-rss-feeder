import { vi } from "vitest";
import type { Feed, Article } from "../../types/feed";

export const mockFeeds: Feed[] = [
  {
    id: "feed-1",
    title: "Tech Blog",
    url: "https://example.com/feed.xml",
    siteUrl: "https://example.com",
    description: "A tech blog",
    iconUrl: null,
    createdAt: "2025-01-01T00:00:00Z",
    updatedAt: "2025-01-01T00:00:00Z",
    lastFetchedAt: null,
    etag: null,
    lastModified: null,
  },
  {
    id: "feed-2",
    title: "News Site",
    url: "https://news.example.com/rss",
    siteUrl: "https://news.example.com",
    description: "Daily news",
    iconUrl: null,
    createdAt: "2025-01-02T00:00:00Z",
    updatedAt: "2025-01-02T00:00:00Z",
    lastFetchedAt: null,
    etag: null,
    lastModified: null,
  },
];

export const mockArticles: Article[] = [
  {
    id: "article-1",
    feedId: "feed-1",
    entryId: "entry-1",
    title: "First Article",
    url: "https://example.com/article-1",
    summary: "Summary of the first article",
    content: null,
    author: "Author One",
    publishedAt: "2025-01-10T00:00:00Z",
    isRead: false,
    readAt: null,
    ogImageUrl: null,
    ogImageLocal: null,
    ogDescription: null,
    ogFetched: false,
    createdAt: "2025-01-10T00:00:00Z",
    feedTitle: "Tech Blog",
  },
  {
    id: "article-2",
    feedId: "feed-1",
    entryId: "entry-2",
    title: "Second Article",
    url: "https://example.com/article-2",
    summary: "Summary of the second article",
    content: null,
    author: "Author Two",
    publishedAt: "2025-01-11T00:00:00Z",
    isRead: true,
    readAt: "2025-01-12T00:00:00Z",
    ogImageUrl: null,
    ogImageLocal: null,
    ogDescription: null,
    ogFetched: false,
    createdAt: "2025-01-11T00:00:00Z",
    feedTitle: "Tech Blog",
  },
  {
    id: "article-3",
    feedId: "feed-2",
    entryId: "entry-3",
    title: "News Article",
    url: "https://news.example.com/article-3",
    summary: "Breaking news summary",
    content: null,
    author: null,
    publishedAt: "2025-01-12T00:00:00Z",
    isRead: false,
    readAt: null,
    ogImageUrl: null,
    ogImageLocal: null,
    ogDescription: null,
    ogFetched: false,
    createdAt: "2025-01-12T00:00:00Z",
    feedTitle: "News Site",
  },
];

export const mockInvoke = vi.fn();
export const mockConvertFileSrc = vi.fn(
  (path: string) => "asset://" + path,
);
export const mockShellOpen = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
  convertFileSrc: (path: string) => mockConvertFileSrc(path),
}));

vi.mock("@tauri-apps/plugin-shell", () => ({
  open: (...args: unknown[]) => mockShellOpen(...args),
}));
