import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  mockInvoke,
  mockArticles,
  mockConvertFileSrc,
} from "../../test/mocks/tauri";
import { useFeedStore } from "../../stores/feedStore";
import { ArticleCard } from "../ArticleCard";
import type { Article } from "../../types/feed";

describe("ArticleCard", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockConvertFileSrc.mockClear();
    useFeedStore.setState({
      feeds: [],
      articles: [...mockArticles],
      selectedFeedId: null,
      loading: false,
      error: null,
    });
  });

  it("記事タイトルと概要を表示する", () => {
    const article = mockArticles[0];
    render(<ArticleCard article={article} />);

    expect(screen.getByText(article.title)).toBeInTheDocument();
    expect(screen.getByText(article.summary!)).toBeInTheDocument();
  });

  it("未読記事にインジケーターが表示される", () => {
    const unreadArticle = mockArticles[0]; // isRead: false
    const readArticle = mockArticles[1]; // isRead: true

    const { rerender } = render(<ArticleCard article={unreadArticle} />);
    expect(screen.getByTestId("unread-indicator")).toBeInTheDocument();

    rerender(<ArticleCard article={readArticle} />);
    expect(screen.queryByTestId("unread-indicator")).not.toBeInTheDocument();
  });

  it("クリックで既読になる", async () => {
    const user = userEvent.setup();
    const article = mockArticles[0]; // isRead: false
    mockInvoke.mockResolvedValueOnce(undefined);

    render(<ArticleCard article={article} />);
    await user.click(screen.getByRole("article"));

    expect(mockInvoke).toHaveBeenCalledWith("mark_article_read", {
      articleId: article.id,
    });
  });

  it("OGP画像がある記事カードに画像が表示される", () => {
    const articleWithOgImage: Article = {
      ...mockArticles[0],
      ogImageUrl: "https://example.com/og-image.jpg",
      ogFetched: true,
    };

    render(<ArticleCard article={articleWithOgImage} />);

    const img = screen.getByRole("img");
    expect(img).toBeInTheDocument();
    expect(img).toHaveAttribute("src", "https://example.com/og-image.jpg");
  });

  it("ローカルキャッシュ画像がある場合convertFileSrcで変換される", () => {
    const articleWithLocalImage: Article = {
      ...mockArticles[0],
      ogImageLocal: "/path/to/local/image.jpg",
      ogImageUrl: "https://example.com/og-image.jpg",
      ogFetched: true,
    };

    render(<ArticleCard article={articleWithLocalImage} />);

    const img = screen.getByRole("img");
    expect(img).toBeInTheDocument();
    expect(img).toHaveAttribute("src", "asset:///path/to/local/image.jpg");
    expect(mockConvertFileSrc).toHaveBeenCalledWith("/path/to/local/image.jpg");
  });

  it("OGP画像がない場合プレースホルダーが表示される", () => {
    const articleWithoutImage: Article = {
      ...mockArticles[0],
      ogImageUrl: null,
      ogImageLocal: null,
      ogFetched: true,
    };

    render(<ArticleCard article={articleWithoutImage} />);

    expect(screen.queryByRole("img")).not.toBeInTheDocument();
    expect(screen.getByTestId("og-placeholder")).toBeInTheDocument();
  });

  it("OGP descriptionがsummaryより優先される", () => {
    const articleWithOgDesc: Article = {
      ...mockArticles[0],
      summary: "Summary of the first article",
      ogDescription: "OGP description takes priority",
      ogFetched: true,
    };

    render(<ArticleCard article={articleWithOgDesc} />);

    expect(
      screen.getByText("OGP description takes priority"),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("Summary of the first article"),
    ).not.toBeInTheDocument();
  });
});
