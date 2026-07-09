export const SHELL_WINDOW_MIN_WIDTH = 420;
export const SHELL_WINDOW_MIN_HEIGHT = 280;
export const SHELL_WINDOW_TASKBAR_HEIGHT = 48;
export const SHELL_WINDOW_MARGIN = 16;

export function clampWindowSize(
  x: number,
  y: number,
  width: number,
  height: number,
): { x: number; y: number; width: number; height: number } {
  const viewportWidth = typeof window !== "undefined" ? window.innerWidth : 1920;
  const viewportHeight =
    typeof window !== "undefined" ? window.innerHeight : 1080;
  const maxWidth = viewportWidth - SHELL_WINDOW_MARGIN * 2;
  const maxHeight =
    viewportHeight - SHELL_WINDOW_TASKBAR_HEIGHT - SHELL_WINDOW_MARGIN * 2;

  const nextWidth = Math.min(
    Math.max(width, SHELL_WINDOW_MIN_WIDTH),
    maxWidth,
  );
  const nextHeight = Math.min(
    Math.max(height, SHELL_WINDOW_MIN_HEIGHT),
    maxHeight,
  );
  const nextX = Math.max(
    SHELL_WINDOW_MARGIN,
    Math.min(x, viewportWidth - nextWidth - SHELL_WINDOW_MARGIN),
  );
  const nextY = Math.max(
    SHELL_WINDOW_MARGIN,
    Math.min(
      y,
      viewportHeight - SHELL_WINDOW_TASKBAR_HEIGHT - nextHeight - SHELL_WINDOW_MARGIN,
    ),
  );

  return {
    x: nextX,
    y: nextY,
    width: nextWidth,
    height: nextHeight,
  };
}

export function clampWindowPosition(
  x: number,
  y: number,
  width: number,
  height: number,
): { x: number; y: number } {
  const bounds = clampWindowSize(x, y, width, height);
  return { x: bounds.x, y: bounds.y };
}
