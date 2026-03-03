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
    });
  });
});
