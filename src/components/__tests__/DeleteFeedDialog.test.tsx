import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { mockInvoke, mockFeeds, mockArticles } from "../../test/mocks/tauri";
import { useFeedStore } from "../../stores/feedStore";
import { DeleteFeedDialog } from "../DeleteFeedDialog";

describe("DeleteFeedDialog", () => {
  const onClose = vi.fn();
  const feed = mockFeeds[0];

  beforeEach(() => {
    mockInvoke.mockReset();
    onClose.mockReset();
    useFeedStore.setState({
      feeds: [...mockFeeds],
      articles: [...mockArticles],
      selectedFeedId: null,
      loading: false,
      error: null,
    });
  });

  it("open=falseの場合は何も表示しない", () => {
    const { container } = render(
      <DeleteFeedDialog open={false} onClose={onClose} feed={feed} />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("フィード名が表示される", () => {
    render(<DeleteFeedDialog open={true} onClose={onClose} feed={feed} />);
    expect(screen.getByText(/Tech Blog/)).toBeInTheDocument();
  });

  it("キャンセルでonCloseが呼ばれる", async () => {
    const user = userEvent.setup();
    render(<DeleteFeedDialog open={true} onClose={onClose} feed={feed} />);

    await user.click(screen.getByText("キャンセル"));

    expect(onClose).toHaveBeenCalled();
  });

  it("削除ボタンでフィードが削除される", async () => {
    const user = userEvent.setup();
    // removeFeedのinvoke + loadArticlesのinvoke
    mockInvoke.mockResolvedValueOnce(undefined);
    mockInvoke.mockResolvedValueOnce([]);

    render(<DeleteFeedDialog open={true} onClose={onClose} feed={feed} />);

    await user.click(screen.getByText("削除"));

    expect(mockInvoke).toHaveBeenCalledWith("remove_feed", {
      feedId: "feed-1",
    });
    expect(onClose).toHaveBeenCalled();
  });
});
