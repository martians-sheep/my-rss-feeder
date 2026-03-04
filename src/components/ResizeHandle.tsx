import { invoke } from "@tauri-apps/api/core";
import { useArticleViewStore } from "../stores/articleViewStore";

const SIDEBAR_WIDTH = 256;
const RESIZE_HANDLE_WIDTH = 4;

export function ResizeHandle() {
  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = useArticleViewStore.getState().articleListWidth;

    // Hide native WebView during drag so mouse events reach the main webview
    invoke("hide_article_webview").catch(console.error);

    const handleMouseMove = (e: MouseEvent) => {
      const delta = e.clientX - startX;
      const newWidth = Math.max(200, Math.min(600, startWidth + delta));
      useArticleViewStore.setState({ articleListWidth: newWidth });
    };

    const handleMouseUp = () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";

      // Update Rust-side bounds and show webview again
      const finalWidth = useArticleViewStore.getState().articleListWidth;
      invoke("update_article_webview_bounds", {
        leftOffset: SIDEBAR_WIDTH + finalWidth + RESIZE_HANDLE_WIDTH,
      })
        .then(() => invoke("show_article_webview"))
        .catch(console.error);
    };

    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
  };

  return (
    <div
      className="w-1 shrink-0 cursor-col-resize bg-gray-200 transition-colors hover:bg-blue-400 active:bg-blue-500"
      onMouseDown={handleMouseDown}
    />
  );
}
