import { useRef } from "react";
import { useFeedStore } from "../stores/feedStore";
import { DateFilter } from "./DateFilter";
import { SortToggle } from "./SortToggle";

export function Toolbar() {
  const refreshAllFeeds = useFeedStore((s) => s.refreshAllFeeds);
  const loading = useFeedStore((s) => s.loading);
  const selectedFeedId = useFeedStore((s) => s.selectedFeedId);
  const feeds = useFeedStore((s) => s.feeds);
  const searchQuery = useFeedStore((s) => s.searchQuery);
  const setSearchQuery = useFeedStore((s) => s.setSearchQuery);

  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleSearchChange = (value: string) => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    debounceRef.current = setTimeout(() => {
      setSearchQuery(value);
    }, 300);
  };

  const selectedFeed = feeds.find((f) => f.id === selectedFeedId);
  const title = selectedFeed ? selectedFeed.title : "すべての記事";

  return (
    <div className="border-b border-gray-200 px-6 py-3">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-bold text-gray-900">{title}</h2>
        <button
          onClick={refreshAllFeeds}
          disabled={loading}
          className="rounded-lg bg-gray-100 px-3 py-1.5 text-sm text-gray-600 transition-colors hover:bg-gray-200 disabled:opacity-50"
        >
          {loading ? "更新中..." : "更新"}
        </button>
      </div>
      <div className="mt-2 flex items-center gap-4">
        <DateFilter />
        <SortToggle />
        <div className="relative ml-auto">
          <svg
            className="pointer-events-none absolute top-1/2 left-2 h-3.5 w-3.5 -translate-y-1/2 text-gray-400"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
            />
          </svg>
          <input
            type="text"
            placeholder="記事を検索..."
            defaultValue={searchQuery}
            onChange={(e) => handleSearchChange(e.target.value)}
            className="w-48 rounded-lg border border-gray-200 py-1 pr-2 pl-7 text-sm text-gray-700 placeholder-gray-400 transition-colors focus:border-blue-400 focus:outline-none"
          />
        </div>
      </div>
    </div>
  );
}
