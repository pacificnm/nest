import type { ShellApp } from "../shell/types";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faBars, faRobot } from "@fortawesome/free-solid-svg-icons";

type TaskbarProps = {
  apps: ShellApp[];
  runningApps: string[];
  focusedApp: string | null;
  onStartToggle: () => void;
  onLaunchApp: (appId: string) => void;
};

export function Taskbar({
  apps,
  runningApps,
  focusedApp,
  onStartToggle,
  onLaunchApp,
}: TaskbarProps) {
  const running = apps.filter((app) => runningApps.includes(app.id));

  return (
    <div className="taskbar">
      <button className="start-button mr-3" onClick={onStartToggle}>
        <FontAwesomeIcon icon={faBars} className="text-lg" />
      </button>

      <div className="flex items-center gap-1">
        {running.map((app) => (
          <div
            key={app.id}
            className="taskbar-item relative"
            onClick={() => onLaunchApp(app.id)}
            title={app.name}
          >
            <FontAwesomeIcon
              icon={app.icon as never}
              className={`text-xl ${focusedApp === app.id ? "text-nest-primary" : "text-nest-muted"}`}
            />
            {focusedApp === app.id && (
              <div className="absolute -bottom-1 left-1/2 h-0.5 w-6 -translate-x-1/2 rounded-full bg-nest-primary" />
            )}
          </div>
        ))}
      </div>

      <div className="ml-auto flex items-center gap-3 pr-3 text-xs text-nest-muted">
        <button
          type="button"
          onClick={() => onLaunchApp("agent")}
          title="Nest Agent"
          aria-label="Nest Agent"
          className={`flex size-6 items-center justify-center rounded-nest-sm hover:bg-nest-muted/10 hover:text-nest-foreground ${
            focusedApp === "agent" ? "text-nest-primary" : "text-nest-muted"
          }`}
        >
          <FontAwesomeIcon icon={faRobot} className="text-base" />
        </button>
        <span className="font-medium">
          {new Date().toLocaleDateString([], {
            weekday: "short",
            month: "short",
            day: "numeric",
          })}
        </span>
        <span className="font-semibold">
          {new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
        </span>
      </div>
    </div>
  );
}
