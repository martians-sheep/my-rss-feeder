import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { NotificationSettings } from "../types/settings";

interface SettingsState {
  notification: NotificationSettings;
  loading: boolean;
  error: string | null;
  loadSettings: () => Promise<void>;
  saveNotificationSettings: (settings: NotificationSettings) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  notification: { enabled: false, time: "10:00" },
  loading: false,
  error: null,

  loadSettings: async () => {
    set({ loading: true, error: null });
    try {
      const notification =
        await invoke<NotificationSettings>("get_notification_settings");
      set({ notification, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  saveNotificationSettings: async (settings: NotificationSettings) => {
    set({ loading: true, error: null });
    try {
      await invoke("save_notification_settings", { settings });
      set({ notification: settings, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
}));
