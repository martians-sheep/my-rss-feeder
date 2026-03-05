import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { mockInvoke } from "../../test/mocks/tauri";
import { SortToggle } from "../SortToggle";
import { useFeedStore } from "../../stores/feedStore";

describe("SortToggle", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockResolvedValue([]);
    useFeedStore.setState({
      feeds: [],
      articles: [],
      selectedFeedId: null,
      dateFilter: { preset: "all", customDate: null },
      sortOrder: "publishedDate",
      loading: false,
      error: null,
    });
  });

  it("ソートボタンが2つ表示される", () => {
    render(<SortToggle />);

    expect(screen.getByText("日付順")).toBeInTheDocument();
    expect(screen.getByText("受信順")).toBeInTheDocument();
  });

  it("デフォルトで日付順が選択されている", () => {
    render(<SortToggle />);

    const publishedBtn = screen.getByText("日付順");
    expect(publishedBtn.className).toContain("bg-blue-600");
  });

  it("受信順ボタンクリックでsortOrderが変更される", async () => {
    const user = userEvent.setup();
    render(<SortToggle />);

    await user.click(screen.getByText("受信順"));

    expect(useFeedStore.getState().sortOrder).toBe("receivedDate");
  });

  it("日付順ボタンクリックでsortOrderが変更される", async () => {
    useFeedStore.setState({ sortOrder: "receivedDate" });
    const user = userEvent.setup();
    render(<SortToggle />);

    await user.click(screen.getByText("日付順"));

    expect(useFeedStore.getState().sortOrder).toBe("publishedDate");
  });
});
