import type { Feed } from "../types/feed";

interface FeedListItemProps {
  feed: Feed;
  selected: boolean;
  onClick: () => void;
  onDelete: (feed: Feed) => void;
}

export function FeedListItem({
  feed,
  selected,
  onClick,
  onDelete,
}: FeedListItemProps) {
  return (
    <button
      onClick={onClick}
      className={`group relative w-full rounded-lg px-3 py-2 text-left text-sm transition-colors ${
        selected
          ? "bg-gray-700 font-medium text-white"
          : "text-gray-300 hover:bg-gray-700/50 hover:text-white"
      }`}
    >
      <span className="block truncate pr-6">{feed.title}</span>
      <span
        role="button"
        aria-label={`${feed.title}を削除`}
        onClick={(e) => {
          e.stopPropagation();
          onDelete(feed);
        }}
        className="absolute top-1/2 right-2 -translate-y-1/2 rounded p-0.5 text-gray-400 opacity-0 transition-opacity hover:text-red-400 group-hover:opacity-100"
      >
        ✕
      </span>
    </button>
  );
}
