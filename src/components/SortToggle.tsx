import { useFeedStore } from "../stores/feedStore";
import type { ArticleSortOrder } from "../types/feed";

const options: { value: ArticleSortOrder; label: string }[] = [
  { value: "publishedDate", label: "日付順" },
  { value: "receivedDate", label: "受信順" },
];

export function SortToggle() {
  const sortOrder = useFeedStore((s) => s.sortOrder);
  const setSortOrder = useFeedStore((s) => s.setSortOrder);

  return (
    <div className="flex items-center gap-1">
      {options.map((o) => (
        <button
          key={o.value}
          onClick={() => setSortOrder(o.value)}
          className={`rounded-md px-3 py-1 text-sm transition-colors ${
            sortOrder === o.value
              ? "bg-blue-600 text-white"
              : "bg-gray-100 text-gray-600 hover:bg-gray-200"
          }`}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}
