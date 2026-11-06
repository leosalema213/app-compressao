export const themes = [
  { value: "system", label: "Sistema" },
  { value: "light", label: "Claro" },
  { value: "dark", label: "Escuro" },
] as const;

export const palettes = [
  { value: "purple", label: "Roxo" },
  { value: "blue", label: "Azul" },
  { value: "green", label: "Verde" },
  { value: "pink", label: "Rosa" },
] as const;

interface Settings {
  version: 1;
  theme: (typeof themes)[number]["value"];
  palette: (typeof palettes)[number]["value"];
  outputFolder: string;
}

export const defaultSettings: Settings = {
  version: 1,
  theme: "system",
  palette: "purple",
  outputFolder: "",
};

const key = "lpstoolkit.settings";
const changeEvent = "lpstoolkit:settings";
let volatileSettings: Settings | undefined;

export function readSettings(): Settings {
  try {
    const raw = window.localStorage.getItem(key);
    const saved: Partial<Record<keyof Settings, unknown>> | null = raw
      ? JSON.parse(raw)
      : {
          theme: window.localStorage.getItem("appearance.theme"),
          palette: window.localStorage.getItem("appearance.palette"),
        };
    if (!saved || (saved.version !== undefined && saved.version !== 1)) {
      return defaultSettings;
    }
    return {
      version: 1,
      theme:
        themes.find(({ value }) => value === saved.theme)?.value ??
        defaultSettings.theme,
      palette:
        palettes.find(({ value }) => value === saved.palette)?.value ??
        defaultSettings.palette,
      outputFolder:
        typeof saved.outputFolder === "string" ? saved.outputFolder : "",
    };
  } catch {
    return volatileSettings ?? defaultSettings;
  }
}

export function saveSettings(changes: Partial<Omit<Settings, "version">>) {
  const settings = { ...readSettings(), ...changes };
  try {
    window.localStorage.setItem(key, JSON.stringify(settings));
  } catch {
    volatileSettings = settings;
  }
  window.dispatchEvent(new CustomEvent(changeEvent, { detail: settings }));
}

export function subscribeToSettings(onChange: (settings: Settings) => void) {
  const receive = (event: Event) =>
    onChange((event as CustomEvent<Settings>).detail);
  window.addEventListener(changeEvent, receive);
  return () => window.removeEventListener(changeEvent, receive);
}
