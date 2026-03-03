import { describe, it, expect, beforeEach } from "vitest";
import { mockInvoke, mockFeeds, mockArticles } from "../../test/mocks/tauri";
import { useFeedStore } from "../feedStore";

describe("feedStore", () => {
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

  it("loadFeedsでフィード一覧が取得される", async () => {
    mockInvoke.mockResolvedValueOnce(mockFeeds);

    await useFeedStore.getState().loadFeeds();

    expect(mockInvoke).toHaveBeenCalledWith("list_feeds");
    expect(useFeedStore.getState().feeds).toEqual(mockFeeds);
  });

  it("addFeedでフィードが追加される", async () => {
    const newFeed = mockFeeds[0];
    mockInvoke.mockResolvedValueOnce(newFeed);

    await useFeedStore.getState().addFeed("https://example.com/feed.xml");

    expect(mockInvoke).toHaveBeenCalledWith("add_feed", {
      url: "https://example.com/feed.xml",
    });
    expect(useFeedStore.getState().feeds).toContainEqual(newFeed);
  });

  it("removeFeedでフィードが削除される", async () => {
    useFeedStore.setState({ feeds: [...mockFeeds] });
    mockInvoke.mockResolvedValueOnce(undefined);

    await useFeedStore.getState().removeFeed("feed-1");

    expect(mockInvoke).toHaveBeenCalledWith("remove_feed", { feedId: "feed-1" });
    expect(useFeedStore.getState().feeds.find((f) => f.id === "feed-1")).toBeUndefined();
  });

  it("selectFeedで選択フィードが変更される", () => {
    useFeedStore.getState().selectFeed("feed-1");

    expect(useFeedStore.getState().selectedFeedId).toBe("feed-1");
  });

  it("markArticleReadで記事が既読になる", async () => {
    useFeedStore.setState({ articles: [...mockArticles] });
    mockInvoke.mockResolvedValueOnce(undefined);

    await useFeedStore.getState().markArticleRead("article-1");

    expect(mockInvoke).toHaveBeenCalledWith("mark_article_read", {
      articleId: "article-1",
    });
    const article = useFeedStore.getState().articles.find((a) => a.id === "article-1");
    expect(article?.isRead).toBe(true);
  });

  it("loadArticlesで記事一覧が取得される", async () => {
    mockInvoke.mockResolvedValueOnce(mockArticles);

    await useFeedStore.getState().loadArticles();

    expect(mockInvoke).toHaveBeenCalledWith("list_articles", { feedId: null });
    expect(useFeedStore.getState().articles).toEqual(mockArticles);
  });

  it("loadArticlesでフィード選択時にフィルタされる", async () => {
    useFeedStore.setState({ selectedFeedId: "feed-1" });
    const feed1Articles = mockArticles.filter((a) => a.feedId === "feed-1");
    mockInvoke.mockResolvedValueOnce(feed1Articles);

    await useFeedStore.getState().loadArticles();

    expect(mockInvoke).toHaveBeenCalledWith("list_articles", { feedId: "feed-1" });
    expect(useFeedStore.getState().articles).toEqual(feed1Articles);
  });

  it("updateArticleOgpで記事のOGP情報が更新される", () => {
    useFeedStore.setState({ articles: [...mockArticles] });

    useFeedStore.getState().updateArticleOgp("article-1", {
      ogImageUrl: "https://example.com/og.jpg",
      ogImageLocal: "/cache/og.jpg",
      ogDescription: "OGP description",
    });

    const article = useFeedStore.getState().articles.find((a) => a.id === "article-1");
    expect(article?.ogImageUrl).toBe("https://example.com/og.jpg");
    expect(article?.ogImageLocal).toBe("/cache/og.jpg");
    expect(article?.ogDescription).toBe("OGP description");
    expect(article?.ogFetched).toBe(true);
  });
});
