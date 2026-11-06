import { useEffect, useState } from "react";
import {
  palettes,
  readSettings,
  saveSettings,
  subscribeToSettings,
  themes,
} from "@/lib/settings";

export function AppearanceControls() {
  const [{ theme, palette }, setSettings] = useState(readSettings);

  useEffect(() => subscribeToSettings(setSettings), []);

  useEffect(() => {
    const root = document.documentElement;
    root.dataset.theme = theme;
    root.dataset.palette = palette;
  }, [theme, palette]);

  return (
    <div className="appearance-controls">
      <div className="palette-picker" role="group" aria-label="Cor de destaque">
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
      <select
        aria-label="Tema"
        value={theme}
        onChange={(event) => {
          const next = themes.find(({ value }) => value === event.target.value);
          if (next) saveSettings({ theme: next.value });
        }}
      >
        {themes.map(({ value, label }) => (
          <option key={value} value={value}>
            {label}
          </option>
        ))}
      </select>
    </div>
  );
}
