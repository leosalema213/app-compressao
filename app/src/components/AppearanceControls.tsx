import { useEffect, useRef, useState } from "react";
import { ChevronDown, Monitor, Moon, Sun } from "lucide-react";
import {
  palettes,
  readSettings,
  saveSettings,
  subscribeToSettings,
  themes,
} from "@/lib/settings";

const themeIcons = { system: Monitor, light: Sun, dark: Moon };

export function AppearanceControls() {
  const [{ theme, palette }, setSettings] = useState(readSettings);
  const [open, setOpen] = useState(false);
  const controls = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const ThemeIcon = themeIcons[theme];

  useEffect(() => subscribeToSettings(setSettings), []);

  useEffect(() => {
    const root = document.documentElement;
    root.dataset.theme = theme;
    root.dataset.palette = palette;
  }, [theme, palette]);

  useEffect(() => {
    if (!open) return;
    function closeOutside(event: PointerEvent) {
      if (
        event.target instanceof Node &&
        !controls.current?.contains(event.target)
      ) {
        setOpen(false);
      }
    }
    document.addEventListener("pointerdown", closeOutside);
    return () => document.removeEventListener("pointerdown", closeOutside);
  }, [open]);

  return (
    <div
      className="appearance-controls"
      ref={controls}
      onKeyDown={(event) => {
        if (open && event.key === "Escape") {
          setOpen(false);
          trigger.current?.focus();
        }
      }}
    >
      <button
        ref={trigger}
        type="button"
        className="theme-trigger"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
      >
        <ThemeIcon size={14} aria-hidden="true" /> Tema{" "}
        <ChevronDown size={12} aria-hidden="true" />
      </button>
      {open && (
        <div className="theme-popup">
          <div role="group" aria-label="Modo de aparência">
            <p>Aparência</p>
            <div className="theme-options">
              {themes.map(({ value, label }) => {
                const Icon = themeIcons[value];
                return (
                  <button
                    key={value}
                    type="button"
                    aria-pressed={theme === value}
                    onClick={() => saveSettings({ theme: value })}
                  >
                    <Icon size={15} aria-hidden="true" /> {label}
                  </button>
                );
              })}
            </div>
          </div>
          <p>Cor principal</p>
          <div
            className="palette-picker"
            role="group"
            aria-label="Cor principal"
          >
            {palettes.map(({ value, label }) => (
              <button
                key={value}
                className="palette-option"
                type="button"
                data-color={value}
                aria-label={label}
                aria-pressed={palette === value}
                title={label}
                onClick={() => saveSettings({ palette: value })}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
