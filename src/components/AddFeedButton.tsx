interface AddFeedButtonProps {
  onClick: () => void;
}

export function AddFeedButton({ onClick }: AddFeedButtonProps) {
  return (
    <button
      onClick={onClick}
      className="w-full rounded-lg border border-dashed border-gray-500 px-3 py-2 text-sm text-gray-400 transition-colors hover:border-gray-300 hover:text-white"
    >
      + フィードを追加
    </button>
  );
}
