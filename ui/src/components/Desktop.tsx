import type { ShellApp } from "../shell/types";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";

type DesktopProps = {
  apps: ShellApp[];
  onLaunchApp: (appId: string) => void;
};

export function Desktop({ apps, onLaunchApp }: DesktopProps) {
  return (
    <div className="nest-desktop">
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6">
        {apps.map((app) => (
          <div
            key={app.id}
            className="desktop-icon"
            onClick={() => onLaunchApp(app.id)}
            onDoubleClick={() => onLaunchApp(app.id)}
          >
            <div className="flex h-16 w-16 items-center justify-center rounded-nest-lg border border-nest-border bg-nest-surface shadow-lg">
              <FontAwesomeIcon icon={app.icon as never} className="text-3xl text-nest-primary" />
            </div>
            <span className="desktop-icon-label">{app.name}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
