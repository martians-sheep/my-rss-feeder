import type { Feed } from "../types/feed";

interface FeedListItemProps {
  feed: Feed;
  selected: boolean;
  onClick: () => void;
}

export function FeedListItem({ feed, selected, onClick }: FeedListItemProps) {
  return (
    <button
      onClick={onClick}
      className={`w-full rounded-lg px-3 py-2 text-left text-sm transition-colors ${
        selected
          ? "bg-gray-700 font-medium text-white"
          : "text-gray-300 hover:bg-gray-700/50 hover:text-white"
      }`}
    >
      {feed.title}
    </button>
  );
}
