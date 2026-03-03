import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Feed, Article } from "../types/feed";

interface OgpData {
  ogImageUrl: string | null;
  ogImageLocal: string | null;
  ogDescription: string | null;
}

interface FeedState {
  feeds: Feed[];
  articles: Article[];
  selectedFeedId: string | null;
  loading: boolean;
  error: string | null;
  loadFeeds: () => Promise<void>;
  addFeed: (url: string) => Promise<void>;
  removeFeed: (feedId: string) => Promise<void>;
  selectFeed: (feedId: string | null) => void;
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

  loadArticles: async () => {
    set({ loading: true, error: null });
    try {
      const { selectedFeedId } = get();
      const articles = await invoke<Article[]>("list_articles", {
        feedId: selectedFeedId,
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
