import { useFeedStore } from "../stores/feedStore";
import type { Feed } from "../types/feed";

interface DeleteFeedDialogProps {
  open: boolean;
  onClose: () => void;
  feed: Feed | null;
}

export function DeleteFeedDialog({
  open,
  onClose,
  feed,
}: DeleteFeedDialogProps) {
  const removeFeed = useFeedStore((s) => s.removeFeed);
  const loadArticles = useFeedStore((s) => s.loadArticles);

  if (!open || !feed) return null;

  const handleDelete = async () => {
    await removeFeed(feed.id);
    await loadArticles();
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-md rounded-xl bg-white p-6 shadow-xl">
        <h2 className="text-lg font-bold text-gray-900">フィードを削除</h2>
        <p className="mt-2 text-sm text-gray-600">
          「{feed.title}」を削除しますか？このフィードの記事もすべて削除されます。
        </p>
        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg px-4 py-2 text-sm text-gray-600 hover:bg-gray-100"
          >
            キャンセル
          </button>
          <button
            type="button"
            onClick={handleDelete}
            className="rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700"
          >
            削除
          </button>
        </div>
      </div>
    </div>
  );
}
