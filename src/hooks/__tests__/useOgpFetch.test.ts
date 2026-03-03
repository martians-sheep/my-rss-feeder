import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { mockInvoke, mockArticles } from "../../test/mocks/tauri";
import { useFeedStore } from "../../stores/feedStore";
import { useOgpFetch } from "../useOgpFetch";

describe("useOgpFetch", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    useFeedStore.setState({
      feeds: [],
      articles: [],
      selectedFeedId: null,
      loading: false,
      error: null,
    });
  });

  it("未取得記事のOGPをバッチ取得してストアを更新する", async () => {
    const articles = [
      { ...mockArticles[0], ogFetched: false },
      { ...mockArticles[1], ogFetched: true },
    ];
    useFeedStore.setState({ articles });

    mockInvoke.mockResolvedValueOnce([
      {
        articleId: "article-1",
        success: true,
        data: {
          ogImageUrl: "https://example.com/og.jpg",
          ogImageLocal: "/cache/og.jpg",
          ogDescription: "Fetched OGP description",
        },
        error: null,
      },
    ]);

    renderHook(() => useOgpFetch());

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("fetch_ogp_batch", {
        articleIds: ["article-1"],
      });
    });

    await waitFor(() => {
      const updated = useFeedStore
        .getState()
        .articles.find((a) => a.id === "article-1");
      expect(updated?.ogImageUrl).toBe("https://example.com/og.jpg");
      expect(updated?.ogImageLocal).toBe("/cache/og.jpg");
      expect(updated?.ogDescription).toBe("Fetched OGP description");
      expect(updated?.ogFetched).toBe(true);
    });
  });

  it("取得失敗時もogFetchedがtrueに更新される", async () => {
    const articles = [{ ...mockArticles[0], ogFetched: false }];
    useFeedStore.setState({ articles });

    mockInvoke.mockResolvedValueOnce([
      {
        articleId: "article-1",
        success: false,
        data: null,
        error: "Failed to fetch",
      },
    ]);

    renderHook(() => useOgpFetch());

    await waitFor(() => {
      const updated = useFeedStore
        .getState()
        .articles.find((a) => a.id === "article-1");
      expect(updated?.ogFetched).toBe(true);
      expect(updated?.ogImageUrl).toBeNull();
    });
  });

  it("全記事がogFetched=trueの場合はfetch_ogp_batchを呼ばない", () => {
    const articles = mockArticles.map((a) => ({ ...a, ogFetched: true }));
    useFeedStore.setState({ articles });

    renderHook(() => useOgpFetch());

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
