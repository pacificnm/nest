export type ShellInteraction = "drag" | "resize" | null;

let activeInteraction: ShellInteraction = null;

export function setShellInteraction(mode: ShellInteraction) {
  activeInteraction = mode;
  document.body.classList.toggle("shell-dragging", mode === "drag");
  document.body.classList.toggle("shell-resizing", mode === "resize");
}

export function clearShellInteraction() {
  setShellInteraction(null);
}

export function currentShellInteraction(): ShellInteraction {
  return activeInteraction;
}
