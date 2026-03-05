import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { mockInvoke, mockFeeds, mockArticles } from "../../test/mocks/tauri";
import { useFeedStore } from "../../stores/feedStore";
import { FeedList } from "../FeedList";

describe("FeedList", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    useFeedStore.setState({
      feeds: [...mockFeeds],
      articles: [...mockArticles],
      selectedFeedId: null,
      loading: false,
      error: null,
    });
  });

  it("フィード選択で記事がフィルタされる", async () => {
    const user = userEvent.setup();
    const feed1Articles = mockArticles.filter((a) => a.feedId === "feed-1");
    mockInvoke.mockResolvedValueOnce(feed1Articles);

    render(<FeedList />);

    await user.click(screen.getByText("Tech Blog"));

    expect(useFeedStore.getState().selectedFeedId).toBe("feed-1");
    expect(mockInvoke).toHaveBeenCalledWith("list_articles", {
      feedId: "feed-1",
      dateFrom: null,
      dateTo: null,
      sortOrder: "publishedDate",
    });
  });

  it("削除ボタンで確認ダイアログが表示される", async () => {
    const user = userEvent.setup();
    render(<FeedList />);

    const deleteButton = screen.getByLabelText("Tech Blogを削除");
    await user.click(deleteButton);

    expect(screen.getByText("フィードを削除")).toBeInTheDocument();
    expect(screen.getByText(/Tech Blog.*削除しますか/)).toBeInTheDocument();
  });

  it("削除確認後にフィードが削除される", async () => {
    const user = userEvent.setup();
    // removeFeedのinvoke + loadArticlesのinvoke
    mockInvoke.mockResolvedValueOnce(undefined);
    mockInvoke.mockResolvedValueOnce([]);

    render(<FeedList />);

    const deleteButton = screen.getByLabelText("Tech Blogを削除");
    await user.click(deleteButton);
    await user.click(screen.getByText("削除"));

    expect(mockInvoke).toHaveBeenCalledWith("remove_feed", {
      feedId: "feed-1",
    });
  });
});
