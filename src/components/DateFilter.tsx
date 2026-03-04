import { useState } from "react";
import { useFeedStore } from "../stores/feedStore";
import type { DatePreset } from "../types/feed";

const presets: { value: DatePreset; label: string }[] = [
  { value: "today", label: "今日" },
  { value: "week", label: "過去7日" },
  { value: "month", label: "過去30日" },
  { value: "all", label: "すべて" },
];

export function DateFilter() {
  const dateFilter = useFeedStore((s) => s.dateFilter);
  const setDatePreset = useFeedStore((s) => s.setDatePreset);
  const setDateFilter = useFeedStore((s) => s.setDateFilter);

  const [showCustom, setShowCustom] = useState(false);
  const [customDate, setCustomDate] = useState("");

  const handlePreset = (preset: DatePreset) => {
    setShowCustom(false);
    setDatePreset(preset);
  };

  const handleToggleCustom = () => {
    setShowCustom((prev) => !prev);
  };

  const handleApplyCustom = () => {
    setDateFilter({
      preset: "custom",
      customDate: customDate || null,
    });
  };

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-1">
        {presets.map((p) => (
          <button
            key={p.value}
            onClick={() => handlePreset(p.value)}
            className={`rounded-md px-3 py-1 text-sm transition-colors ${
              dateFilter.preset === p.value && !showCustom
                ? "bg-blue-600 text-white"
                : "bg-gray-100 text-gray-600 hover:bg-gray-200"
            }`}
          >
            {p.label}
          </button>
        ))}
        <button
          onClick={handleToggleCustom}
          className={`ml-1 rounded-md px-2 py-1 text-sm transition-colors ${
            dateFilter.preset === "custom" || showCustom
              ? "bg-blue-600 text-white"
              : "bg-gray-100 text-gray-600 hover:bg-gray-200"
          }`}
          title="日付を指定"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-4 w-4"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
            />
          </svg>
        </button>
      </div>
      {showCustom && (
        <div className="flex items-center gap-2">
          <input
            type="date"
            value={customDate}
            onChange={(e) => setCustomDate(e.target.value)}
            className="rounded-md border border-gray-300 px-2 py-1 text-sm"
          />
          <button
            onClick={handleApplyCustom}
            className="rounded-md bg-blue-600 px-3 py-1 text-sm text-white transition-colors hover:bg-blue-700"
          >
            適用
          </button>
        </div>
      )}
    </div>
  );
}
