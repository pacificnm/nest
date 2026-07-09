import { useRef, useState, type PointerEvent, type ReactNode } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faMinus, faXmark } from "@fortawesome/free-solid-svg-icons";

import { clearShellInteraction, setShellInteraction } from "../../shell/interaction";
import { clampWindowPosition, clampWindowSize } from "../../shell/windowBounds";
import type { ShellWindow } from "../../shell/types";

type ResizeEdge = "east" | "south" | "south-east";

type LiveBounds = {
  x: number;
  y: number;
  width: number;
  height: number;
};

type WindowFrameProps = {
  window: ShellWindow;
  focused: boolean;
  onFocus: () => void;
  onClose: () => void;
  onMinimize: () => void;
  onMove: (x: number, y: number) => void;
  onResize: (width: number, height: number) => void;
  children: ReactNode;
};

export function WindowFrame({
  window,
  focused,
  onFocus,
  onClose,
  onMinimize,
  onMove,
  onResize,
  children,
}: WindowFrameProps) {
  const dragRef = useRef<{
    startX: number;
    startY: number;
    originX: number;
    originY: number;
  } | null>(null);
  const resizeRef = useRef<{
    edge: ResizeEdge;
    startX: number;
    startY: number;
    originWidth: number;
    originHeight: number;
    originX: number;
    originY: number;
  } | null>(null);
  const [liveBounds, setLiveBounds] = useState<LiveBounds | null>(null);
  const liveBoundsRef = useRef<LiveBounds | null>(null);

  const updateLiveBounds = (next: LiveBounds) => {
    liveBoundsRef.current = next;
    setLiveBounds(next);
  };

  const clearLiveBounds = () => {
    liveBoundsRef.current = null;
    setLiveBounds(null);
  };

  if (window.minimized) {
    return null;
  }

  const bounds = liveBounds ?? {
    x: window.x,
    y: window.y,
    width: window.width,
    height: window.height,
  };

  const beginBounds = (): LiveBounds => ({
    x: window.x,
    y: window.y,
    width: window.width,
    height: window.height,
  });

  const onTitlePointerDown = (event: PointerEvent<HTMLDivElement>) => {
    if ((event.target as HTMLElement).closest("button")) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    onFocus();
    const start = beginBounds();
    updateLiveBounds(start);
    setShellInteraction("drag");
    dragRef.current = {
      startX: event.clientX,
      startY: event.clientY,
      originX: start.x,
      originY: start.y,
    };
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const onTitlePointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (!dragRef.current) {
      return;
    }
    event.preventDefault();
    const dx = event.clientX - dragRef.current.startX;
    const dy = event.clientY - dragRef.current.startY;
    const position = clampWindowPosition(
      dragRef.current.originX + dx,
      dragRef.current.originY + dy,
      bounds.width,
      bounds.height,
    );
    setLiveBounds((current) => {
      const base = current ?? { ...beginBounds(), ...position };
      const next = { ...base, x: position.x, y: position.y };
      liveBoundsRef.current = next;
      return next;
    });
  };

  const onTitlePointerUp = (event: PointerEvent<HTMLDivElement>) => {
    if (dragRef.current && liveBoundsRef.current) {
      onMove(liveBoundsRef.current.x, liveBoundsRef.current.y);
    }
    dragRef.current = null;
    clearLiveBounds();
    clearShellInteraction();
    event.currentTarget.releasePointerCapture(event.pointerId);
  };

  const onResizePointerDown =
    (edge: ResizeEdge) => (event: PointerEvent<HTMLDivElement>) => {
      event.preventDefault();
      event.stopPropagation();
      onFocus();
      const start = beginBounds();
      updateLiveBounds(start);
      setShellInteraction("resize");
      resizeRef.current = {
        edge,
        startX: event.clientX,
        startY: event.clientY,
        originWidth: start.width,
        originHeight: start.height,
        originX: start.x,
        originY: start.y,
      };
      event.currentTarget.setPointerCapture(event.pointerId);
    };

  const onResizePointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (!resizeRef.current) {
      return;
    }
    event.preventDefault();
    const dx = event.clientX - resizeRef.current.startX;
    const dy = event.clientY - resizeRef.current.startY;
    const { edge, originWidth, originHeight, originX, originY } = resizeRef.current;

    const nextWidth = edge === "south" ? originWidth : Math.round(originWidth + dx);
    const nextHeight = edge === "east" ? originHeight : Math.round(originHeight + dy);
    const sized = clampWindowSize(originX, originY, nextWidth, nextHeight);

    updateLiveBounds({
      x: sized.x,
      y: sized.y,
      width: sized.width,
      height: sized.height,
    });
  };

  const onResizePointerUp = (event: PointerEvent<HTMLDivElement>) => {
    if (resizeRef.current && liveBoundsRef.current) {
      const finalBounds = liveBoundsRef.current;
      onResize(finalBounds.width, finalBounds.height);
      if (finalBounds.x !== window.x || finalBounds.y !== window.y) {
        onMove(finalBounds.x, finalBounds.y);
      }
    }
    resizeRef.current = null;
    clearLiveBounds();
    clearShellInteraction();
    event.currentTarget.releasePointerCapture(event.pointerId);
  };

  const onFramePointerDown = (event: PointerEvent<HTMLDivElement>) => {
    if ((event.target as HTMLElement).closest(".shell-window-resize")) {
      return;
    }
    onFocus();
  };

  return (
    <div
      className={[
        "shell-window",
        focused ? "shell-window-focused" : "shell-window-unfocused",
        liveBounds ? "shell-window-interacting" : "",
      ].join(" ")}
      style={{
        left: bounds.x,
        top: bounds.y,
        width: bounds.width,
        height: bounds.height,
        zIndex: window.zIndex,
      }}
      onPointerDown={onFramePointerDown}
    >
      <div
        className="shell-window-titlebar"
        onPointerDown={onTitlePointerDown}
        onPointerMove={onTitlePointerMove}
        onPointerUp={onTitlePointerUp}
        onPointerCancel={onTitlePointerUp}
      >
        <span className="shell-window-title">{window.title}</span>
        <div className="shell-window-controls">
          <button
            type="button"
            className="shell-window-control"
            aria-label="Minimize"
            onPointerDown={(event) => event.stopPropagation()}
            onClick={(event) => {
              event.stopPropagation();
              onMinimize();
            }}
          >
            <FontAwesomeIcon icon={faMinus} />
          </button>
          <button
            type="button"
            className="shell-window-control shell-window-control-close"
            aria-label="Close"
            onPointerDown={(event) => event.stopPropagation()}
            onClick={(event) => {
              event.stopPropagation();
              onClose();
            }}
          >
            <FontAwesomeIcon icon={faXmark} />
          </button>
        </div>
      </div>
      <div className="shell-window-content">{children}</div>

      <div
        className="shell-window-resize shell-window-resize-east"
        onPointerDown={onResizePointerDown("east")}
        onPointerMove={onResizePointerMove}
        onPointerUp={onResizePointerUp}
        onPointerCancel={onResizePointerUp}
      />
      <div
        className="shell-window-resize shell-window-resize-south"
        onPointerDown={onResizePointerDown("south")}
        onPointerMove={onResizePointerMove}
        onPointerUp={onResizePointerUp}
        onPointerCancel={onResizePointerUp}
      />
      <div
        className="shell-window-resize shell-window-resize-south-east"
        onPointerDown={onResizePointerDown("south-east")}
        onPointerMove={onResizePointerMove}
        onPointerUp={onResizePointerUp}
        onPointerCancel={onResizePointerUp}
        aria-hidden
      />
    </div>
  );
}
