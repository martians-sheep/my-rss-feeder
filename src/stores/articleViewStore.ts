import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Article } from "../types/feed";

const SIDEBAR_WIDTH = 256;
const RESIZE_HANDLE_WIDTH = 4;

interface ArticleViewState {
  selectedArticle: Article | null;
  loading: boolean;
  articleListWidth: number;
  highlightEnabled: boolean;
  selectArticle: (article: Article) => Promise<void>;
  closeWebview: () => Promise<void>;
  setArticleListWidth: (width: number) => void;
  toggleHighlight: () => Promise<void>;
}

export const useArticleViewStore = create<ArticleViewState>((set, get) => ({
  selectedArticle: null,
  loading: false,
  articleListWidth: 320,
  highlightEnabled: true,

  selectArticle: async (article: Article) => {
    if (!article.url) return;
    set({ selectedArticle: article, loading: true });
    try {
      const { highlightEnabled } = get();
      await invoke("open_article_webview", {
        url: article.url,
        title: highlightEnabled ? article.title : null,
      });
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
    set({ selectedArticle: null, loading: false, highlightEnabled: true });
  },

  setArticleListWidth: (width: number) => {
    set({ articleListWidth: width });
    invoke("update_article_webview_bounds", {
      leftOffset: SIDEBAR_WIDTH + width + RESIZE_HANDLE_WIDTH,
    }).catch((e) => console.error("Failed to update webview bounds:", e));
  },

  toggleHighlight: async () => {
    const { highlightEnabled, selectedArticle } = get();
    const newEnabled = !highlightEnabled;
    set({ highlightEnabled: newEnabled });

    if (!selectedArticle) return;

    try {
      if (newEnabled && selectedArticle.title) {
        await invoke("highlight_in_webview", {
          title: selectedArticle.title,
        });
      } else {
        await invoke("remove_highlight_in_webview");
      }
    } catch (e) {
      console.error("Failed to toggle highlight:", e);
    }
  },
}));
