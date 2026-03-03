import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { mockInvoke, mockFeeds } from "../../test/mocks/tauri";
import { useFeedStore } from "../../stores/feedStore";
import { AddFeedDialog } from "../AddFeedDialog";

describe("AddFeedDialog", () => {
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

  it("URL入力して追加できる", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    mockInvoke.mockResolvedValueOnce(mockFeeds[0]);

    render(<AddFeedDialog open={true} onClose={onClose} />);

    const input = screen.getByPlaceholderText("https://example.com/feed.xml");
    await user.type(input, "https://example.com/feed.xml");
    await user.click(screen.getByRole("button", { name: "追加" }));

    expect(mockInvoke).toHaveBeenCalledWith("add_feed", {
      url: "https://example.com/feed.xml",
    });
    expect(onClose).toHaveBeenCalled();
  });

  it("open=falseのとき表示されない", () => {
    render(<AddFeedDialog open={false} onClose={() => {}} />);

    expect(screen.queryByPlaceholderText("https://example.com/feed.xml")).not.toBeInTheDocument();
  });
});
