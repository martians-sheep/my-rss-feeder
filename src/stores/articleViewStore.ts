import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Article } from "../types/feed";

const SIDEBAR_WIDTH = 256;
const RESIZE_HANDLE_WIDTH = 4;

interface ArticleViewState {
  selectedArticle: Article | null;
  loading: boolean;
  articleListWidth: number;
  selectArticle: (article: Article) => Promise<void>;
  closeWebview: () => Promise<void>;
  setArticleListWidth: (width: number) => void;
}

export const useArticleViewStore = create<ArticleViewState>((set) => ({
  selectedArticle: null,
  loading: false,
  articleListWidth: 320,

  selectArticle: async (article: Article) => {
    if (!article.url) return;
    set({ selectedArticle: article, loading: true });
    try {
      await invoke("open_article_webview", { url: article.url });
    } catch (e) {
      console.error("Failed to open article webview:", e);
    } finally {
      set({ loading: false });
    }
  },

  closeWebview: async () => {
    try {
      await invoke("close_article_webview");
    } catch (e) {
      console.error("Failed to close article webview:", e);
    }
    set({ selectedArticle: null, loading: false });
  },

  setArticleListWidth: (width: number) => {
    set({ articleListWidth: width });
    invoke("update_article_webview_bounds", {
      leftOffset: SIDEBAR_WIDTH + width + RESIZE_HANDLE_WIDTH,
    }).catch((e) => console.error("Failed to update webview bounds:", e));
  },
}));
