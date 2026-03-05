import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { useSettingsStore } from "../stores/settingsStore";

interface SettingsDialogProps {
  open: boolean;
  onClose: () => void;
}

export function SettingsDialog({ open, onClose }: SettingsDialogProps) {
  const notification = useSettingsStore((s) => s.notification);
  const saveNotificationSettings = useSettingsStore(
    (s) => s.saveNotificationSettings,
  );

  const [enabled, setEnabled] = useState(notification.enabled);
  const [time, setTime] = useState(notification.time);
  const [version, setVersion] = useState("");

  useEffect(() => {
    if (open) {
      setEnabled(notification.enabled);
      setTime(notification.time);
      getVersion().then(setVersion);
    }
  }, [open, notification]);

  if (!open) return null;

  const handleSave = async () => {
    await saveNotificationSettings({ enabled, time });
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-md rounded-xl bg-white p-6 shadow-xl">
        <h2 className="text-lg font-bold text-gray-900">設定</h2>

        <div className="mt-4 space-y-4">
          <div className="flex items-center justify-between">
            <label
              htmlFor="notification-toggle"
              className="text-sm font-medium text-gray-700"
            >
              日次通知
            </label>
            <button
              id="notification-toggle"
              type="button"
              role="switch"
              aria-checked={enabled}
              onClick={() => setEnabled(!enabled)}
              className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out ${
                enabled ? "bg-blue-600" : "bg-gray-200"
              }`}
            >
              <span
                className={`pointer-events-none inline-block h-5 w-5 rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
                  enabled ? "translate-x-5" : "translate-x-0"
                }`}
              />
            </button>
          </div>

          <div>
            <label
              htmlFor="notification-time"
              className="block text-sm font-medium text-gray-700"
            >
              通知時刻
            </label>
            <input
              id="notification-time"
              type="time"
              value={time}
              onChange={(e) => setTime(e.target.value)}
              disabled={!enabled}
              className="mt-1 w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:ring-1 focus:ring-blue-500 focus:outline-none disabled:bg-gray-100 disabled:text-gray-400"
            />
          </div>
        </div>

        {version && (
          <p className="mt-6 text-xs text-gray-400">バージョン {version}</p>
        )}

        <div className="mt-6 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg px-4 py-2 text-sm text-gray-600 hover:bg-gray-100"
          >
            キャンセル
          </button>
          <button
            type="button"
            onClick={handleSave}
            className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
          >
            保存
          </button>
        </div>
      </div>
    </div>
  );
}
