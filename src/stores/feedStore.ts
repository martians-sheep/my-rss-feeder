import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type {
  Feed,
  Article,
  DateFilter,
  DatePreset,
  ArticleSortOrder,
} from "../types/feed";

interface OgpData {
  ogImageUrl: string | null;
  ogImageLocal: string | null;
  ogDescription: string | null;
}

function localMidnight(date: Date): Date {
  const d = new Date(date);
  d.setHours(0, 0, 0, 0);
  return d;
}

function toRfc3339(date: Date): string {
  return date.toISOString().replace("Z", "+00:00");
}

function computeDateRange(filter: DateFilter): {
  dateFrom: string | null;
  dateTo: string | null;
} {
  if (filter.preset === "all") {
    return { dateFrom: null, dateTo: null };
  }
  if (filter.preset === "custom") {
    if (!filter.customDate) {
      return { dateFrom: null, dateTo: null };
    }
    // "YYYY-MM-DD" をローカルタイムゾーンの0時として解釈
    const start = new Date(filter.customDate + "T00:00:00");
    const end = new Date(start.getTime() + 24 * 60 * 60 * 1000);
    return { dateFrom: toRfc3339(start), dateTo: toRfc3339(end) };
  }
  const now = new Date();
  const start = localMidnight(now);
  if (filter.preset === "week") {
    start.setDate(start.getDate() - 6);
  } else if (filter.preset === "month") {
    start.setDate(start.getDate() - 29);
  }
  return { dateFrom: toRfc3339(start), dateTo: null };
}

interface FeedState {
  feeds: Feed[];
  articles: Article[];
  selectedFeedId: string | null;
  dateFilter: DateFilter;
  sortOrder: ArticleSortOrder;
  searchQuery: string;
  loading: boolean;
  error: string | null;
  loadFeeds: () => Promise<void>;
  addFeed: (url: string) => Promise<void>;
  removeFeed: (feedId: string) => Promise<void>;
  selectFeed: (feedId: string | null) => void;
  setDatePreset: (preset: DatePreset) => void;
  setDateFilter: (filter: DateFilter) => void;
  setSortOrder: (sortOrder: ArticleSortOrder) => void;
  setSearchQuery: (query: string) => void;
  loadArticles: () => Promise<void>;
  markArticleRead: (articleId: string) => Promise<void>;
  refreshFeed: (feedId: string) => Promise<void>;
  refreshAllFeeds: () => Promise<void>;
  updateArticleOgp: (articleId: string, ogpData: OgpData) => void;
}

export const useFeedStore = create<FeedState>((set, get) => ({
  feeds: [],
  articles: [],
  selectedFeedId: null,
  dateFilter: { preset: "all", customDate: null },
  sortOrder: "publishedDate",
  searchQuery: "",
  loading: false,
  error: null,

  loadFeeds: async () => {
    set({ loading: true, error: null });
    try {
      const feeds = await invoke<Feed[]>("list_feeds");
      set({ feeds, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  addFeed: async (url: string) => {
    set({ loading: true, error: null });
    try {
      const feed = await invoke<Feed>("add_feed", { url });
      set((state) => ({ feeds: [...state.feeds, feed], loading: false }));
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  removeFeed: async (feedId: string) => {
    set({ loading: true, error: null });
    try {
      await invoke("remove_feed", { feedId });
      set((state) => ({
        feeds: state.feeds.filter((f) => f.id !== feedId),
        selectedFeedId:
          state.selectedFeedId === feedId ? null : state.selectedFeedId,
        loading: false,
      }));
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  selectFeed: (feedId: string | null) => {
    set({ selectedFeedId: feedId });
  },

  setDatePreset: (preset: DatePreset) => {
    const filter: DateFilter = { preset, customDate: null };
    set({ dateFilter: filter });
    get().loadArticles();
  },

  setDateFilter: (filter: DateFilter) => {
    set({ dateFilter: filter });
    get().loadArticles();
  },

  setSortOrder: (sortOrder: ArticleSortOrder) => {
    set({ sortOrder });
    get().loadArticles();
  },

  setSearchQuery: (query: string) => {
    set({ searchQuery: query });
    get().loadArticles();
  },

  loadArticles: async () => {
    set({ loading: true, error: null });
    try {
      const { selectedFeedId, dateFilter, sortOrder, searchQuery } = get();
      const { dateFrom, dateTo } = computeDateRange(dateFilter);
      const articles = await invoke<Article[]>("list_articles", {
        feedId: selectedFeedId,
        dateFrom,
        dateTo,
        query: searchQuery || null,
        sortOrder,
      });
      set({ articles, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  markArticleRead: async (articleId: string) => {
    try {
      await invoke("mark_article_read", { articleId });
      set((state) => ({
        articles: state.articles.map((a) =>
          a.id === articleId ? { ...a, isRead: true } : a,
        ),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  refreshFeed: async (feedId: string) => {
    set({ loading: true, error: null });
    try {
      await invoke("refresh_feed", { feedId });
      await get().loadArticles();
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  refreshAllFeeds: async () => {
    set({ loading: true, error: null });
    try {
      await invoke("refresh_all_feeds");
      await get().loadFeeds();
      await get().loadArticles();
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  updateArticleOgp: (articleId: string, ogpData: OgpData) => {
    set((state) => ({
      articles: state.articles.map((a) =>
        a.id === articleId
          ? { ...a, ...ogpData, ogFetched: true }
          : a,
      ),
    }));
  },
}));
