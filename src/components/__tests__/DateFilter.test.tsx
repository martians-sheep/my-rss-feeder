import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { mockInvoke } from "../../test/mocks/tauri";
import { DateFilter } from "../DateFilter";
import { useFeedStore } from "../../stores/feedStore";

describe("DateFilter", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockResolvedValue([]);
    useFeedStore.setState({
      feeds: [],
      articles: [],
      selectedFeedId: null,
      dateFilter: { preset: "all", customDate: null },
      loading: false,
      error: null,
    });
  });

  it("プリセットボタンが4つ表示される", () => {
    render(<DateFilter />);

    expect(screen.getByText("今日")).toBeInTheDocument();
    expect(screen.getByText("過去7日")).toBeInTheDocument();
    expect(screen.getByText("過去30日")).toBeInTheDocument();
    expect(screen.getByText("すべて")).toBeInTheDocument();
  });

  it("プリセットボタンクリックでsetDatePresetが呼ばれる", async () => {
    const user = userEvent.setup();
    render(<DateFilter />);

    await user.click(screen.getByText("今日"));

    expect(useFeedStore.getState().dateFilter.preset).toBe("today");
  });

  it("カレンダーボタンクリックでカスタム日付入力が表示される", async () => {
    const user = userEvent.setup();
    render(<DateFilter />);

    expect(screen.queryByText("適用")).not.toBeInTheDocument();

    await user.click(screen.getByTitle("日付を指定"));

    expect(screen.getByText("適用")).toBeInTheDocument();
  });
});
