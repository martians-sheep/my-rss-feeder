import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { ask } from "@tauri-apps/plugin-dialog";
import { useEffect } from "react";

async function checkForUpdates() {
  try {
    const update = await check();
    if (!update) return;

    const shouldUpdate = await ask(
      `新しいバージョン ${update.version} が利用可能です。\n\n` +
        `更新内容:\n${update.body ?? "(詳細なし)"}\n\n` +
        `今すぐ更新しますか？`,
      {
        title: "アップデートのお知らせ",
        kind: "info",
        okLabel: "更新する",
        cancelLabel: "後で",
      },
    );

    if (shouldUpdate) {
      await update.downloadAndInstall();
      await relaunch();
    }
  } catch (error) {
    console.error("更新チェックに失敗しました:", error);
  }
}

export function useAppUpdater() {
  useEffect(() => {
    checkForUpdates();
  }, []);
}
