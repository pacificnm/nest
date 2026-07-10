#!/usr/bin/env node
/**
 * Starts the Nest Shell Vite server plus embed-mode app dev servers.
 * Writes ui/.embed-dev-ports.json so Rust launch resolution uses the same ports.
 */
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import concurrently from "concurrently";

const __dirname = dirname(fileURLToPath(import.meta.url));
const UI_ROOT = join(__dirname, "..");
const NEST_ROOT = join(UI_ROOT, "..");
const APPS_DIR = join(NEST_ROOT, "apps");
const SHELL_PORT = 5173;
const BASE_EMBED_PORT = 5174;
const PORTS_FILE = join(UI_ROOT, ".embed-dev-ports.json");

function parseShellSection(manifestText) {
  const launchMatch = manifestText.match(/\[shell\.launch\]([\s\S]*?)(?:\n\[|$)/);
  const devMatch = manifestText.match(/\[shell\.dev\]([\s\S]*?)(?:\n\[|$)/);

  const readValue = (section, key) => {
    if (!section) {
      return null;
    }
    const match = section[1].match(new RegExp(`^${key}\\s*=\\s*["']?([^"'\n#]+)`, "m"));
    return match ? match[1].trim() : null;
  };

  return {
    launchMode: readValue(launchMatch, "mode"),
    devPort: readValue(devMatch, "port"),
  };
}

function inferLaunchMode(launchMode) {
  if (launchMode) {
    return launchMode.toLowerCase();
  }
  return "module";
}

function discoverEmbedApps() {
  if (!existsSync(APPS_DIR)) {
    return [];
  }

  const apps = [];
  for (const entry of readdirSync(APPS_DIR, { withFileTypes: true })) {
    if (!entry.isDirectory() || entry.name.startsWith(".")) {
      continue;
    }

    const appRoot = join(APPS_DIR, entry.name);
    const manifestPath = join(appRoot, "nest-app.toml");
    if (!existsSync(manifestPath)) {
      continue;
    }

    const manifestText = readFileSync(manifestPath, "utf8");
    const { launchMode, devPort } = parseShellSection(manifestText);
    const mode = inferLaunchMode(launchMode);
    if (mode !== "embed") {
      continue;
    }

    const uiRoot = join(appRoot, "ui");
    if (!existsSync(join(uiRoot, "package.json"))) {
      console.warn(`[dev-orchestrator] ${entry.name}: embed mode requires ui/package.json`);
      continue;
    }

    apps.push({
      id: entry.name,
      path: join("apps", entry.name),
      uiRoot,
      devPort: devPort ? Number(devPort) : null,
    });
  }

  apps.sort((left, right) => left.id.localeCompare(right.id));
  return apps;
}

function assignPorts(apps) {
  const assigned = {};
  const used = new Set([SHELL_PORT]);

  for (const app of apps) {
    let port = app.devPort ?? BASE_EMBED_PORT;
    if (used.has(port)) {
      const requested = port;
      while (used.has(port)) {
        port += 1;
      }
      if (app.devPort) {
        console.warn(
          `[dev-orchestrator] ${app.id}: port ${requested} is taken, using ${port}`,
        );
      }
    }
    used.add(port);
    assigned[app.id] = port;
  }
  return assigned;
}

function writePortsFile(ports) {
  writeFileSync(PORTS_FILE, `${JSON.stringify(ports, null, 2)}\n`, "utf8");
}

function main() {
  const embedApps = discoverEmbedApps();
  const ports = assignPorts(embedApps);
  writePortsFile(ports);

  const commands = [
    {
      name: "shell",
      command: `npm run dev -- --port ${SHELL_PORT} --strictPort`,
      cwd: UI_ROOT,
      prefixColor: "cyan",
    },
    ...embedApps.map((app) => ({
      name: app.id,
      command: `npm run dev -- --port ${ports[app.id]} --strictPort`,
      cwd: app.uiRoot,
      env: {
        NEST_DEV_PORT: String(ports[app.id]),
        NEST_SHELL_EMBED: "1",
      },
      prefixColor: "green",
    })),
  ];

  if (commands.length === 1) {
    console.log("No embed apps found — starting shell Vite only.");
  } else {
    console.log(
      `Starting shell :${SHELL_PORT} and ${embedApps.length} embed app dev server(s):`,
    );
    for (const app of embedApps) {
      console.log(`  ${app.id} → http://localhost:${ports[app.id]}`);
    }
  }

  const { result } = concurrently(commands, {
    prefix: "name",
    killOthersOn: ["failure"],
    restartTries: 0,
  });

  result.catch((error) => {
    console.error(error);
    process.exit(1);
  });
}

mkdirSync(UI_ROOT, { recursive: true });
main();
