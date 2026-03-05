import { useState } from "react";
import { useFeedStore } from "../stores/feedStore";
import { FeedListItem } from "./FeedListItem";
import { DeleteFeedDialog } from "./DeleteFeedDialog";
import type { Feed } from "../types/feed";

export function FeedList() {
  const feeds = useFeedStore((s) => s.feeds);
  const selectedFeedId = useFeedStore((s) => s.selectedFeedId);
  const selectFeed = useFeedStore((s) => s.selectFeed);
  const loadArticles = useFeedStore((s) => s.loadArticles);

  const [deletingFeed, setDeletingFeed] = useState<Feed | null>(null);

  const handleSelect = async (feedId: string | null) => {
    selectFeed(feedId);
    // loadArticles reads selectedFeedId from state, but selectFeed just set it
    // We need to wait for the state update, so we call loadArticles after selectFeed
    // Since Zustand updates are sync, this works correctly
    await loadArticles();
  };

  return (
    <>
      <nav className="flex flex-col gap-1">
        <button
          onClick={() => handleSelect(null)}
          className={`w-full rounded-lg px-3 py-2 text-left text-sm transition-colors ${
            selectedFeedId === null
              ? "bg-gray-700 font-medium text-white"
              : "text-gray-300 hover:bg-gray-700/50 hover:text-white"
          }`}
        >
          すべての記事
        </button>
        {feeds.map((feed) => (
          <FeedListItem
            key={feed.id}
            feed={feed}
            selected={selectedFeedId === feed.id}
            onClick={() => handleSelect(feed.id)}
            onDelete={setDeletingFeed}
          />
        ))}
      </nav>
      <DeleteFeedDialog
        open={deletingFeed !== null}
        onClose={() => setDeletingFeed(null)}
        feed={deletingFeed}
      />
    </>
  );
}
