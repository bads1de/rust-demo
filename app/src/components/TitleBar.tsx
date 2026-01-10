import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { X, Square, Minus, Activity } from "lucide-react";

export const TitleBar = () => {
  const [isMaximized, setIsMaximized] = useState(false);
  const appWindow = getCurrentWindow();

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let isMounted = true;

    const setupListener = async () => {
      const maximized = await appWindow.isMaximized();
      if (isMounted) setIsMaximized(maximized);

      // Listen for resize events to update isMaximized state
      unlisten = await appWindow.onResized(async () => {
        const currentlyMaximized = await appWindow.isMaximized();
        if (isMounted) setIsMaximized(currentlyMaximized);
      });
    };

    setupListener();

    return () => {
      isMounted = false;
      if (unlisten) unlisten();
    };
  }, [appWindow]);

  const handleMinimize = () => appWindow.minimize();
  const handleMaximize = async () => {
    await appWindow.toggleMaximize();
    setIsMaximized(await appWindow.isMaximized());
  };
  const handleClose = () => appWindow.close();

  const handleDragStart = (e: React.MouseEvent) => {
    // Only start dragging if the target is the titlebar itself (not buttons)
    if (e.buttons === 1 && e.target === e.currentTarget) {
      if (e.detail === 2) {
        appWindow.toggleMaximize();
      } else {
        appWindow.startDragging();
      }
    }
  };

  return (
    <div
      onMouseDown={handleDragStart}
      className="h-9 bg-linear-to-r from-[#0B0E14] to-[#111620] flex items-center justify-between px-4 select-none fixed top-0 left-0 right-0 z-[100] border-b border-white/5 shadow-sm"
    >
      <div className="flex items-center gap-2 pointer-events-none opacity-80">
        <Activity size={14} className="text-indigo-400" />
        <span className="text-[10px] font-bold text-gray-300 tracking-[0.2em] uppercase">
          Rust-Chart <span className="text-indigo-500">PRO</span>
        </span>
      </div>

      <div className="flex items-center gap-1.5 pointer-events-auto">
        <button
          onClick={handleMinimize}
          className="p-1.5 text-gray-500 hover:text-white hover:bg-white/10 rounded-md transition-all"
        >
          <Minus size={12} />
        </button>
        <button
          onClick={handleMaximize}
          className="p-1.5 text-gray-500 hover:text-white hover:bg-white/10 rounded-md transition-all"
        >
          <Square
            size={10}
            className={isMaximized ? "fill-current opacity-50" : ""}
          />
        </button>
        <button
          onClick={handleClose}
          className="p-1.5 text-gray-500 hover:text-white hover:bg-red-500/80 rounded-md transition-all"
        >
          <X size={14} />
        </button>
      </div>
    </div>
  );
};
