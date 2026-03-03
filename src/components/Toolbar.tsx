import { useFeedStore } from "../stores/feedStore";

export function Toolbar() {
  const refreshAllFeeds = useFeedStore((s) => s.refreshAllFeeds);
  const loading = useFeedStore((s) => s.loading);
  const selectedFeedId = useFeedStore((s) => s.selectedFeedId);
  const feeds = useFeedStore((s) => s.feeds);

  const selectedFeed = feeds.find((f) => f.id === selectedFeedId);
  const title = selectedFeed ? selectedFeed.title : "すべての記事";

  return (
    <div className="flex items-center justify-between border-b border-gray-200 px-6 py-4">
      <h2 className="text-lg font-bold text-gray-900">{title}</h2>
      <button
        onClick={refreshAllFeeds}
        disabled={loading}
        className="rounded-lg bg-gray-100 px-3 py-1.5 text-sm text-gray-600 transition-colors hover:bg-gray-200 disabled:opacity-50"
      >
        {loading ? "更新中..." : "更新"}
      </button>
    </div>
  );
}
