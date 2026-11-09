// @vitest-environment jsdom
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { readSettings, saveSettings } from "@/lib/settings";
import { SettingsBackup } from "./SettingsBackup";

afterEach(() => {
  cleanup();
  clearMocks();
  window.localStorage.clear();
});

test("importação aguarda confirmação e recupera aparência quando a pasta antiga está indisponível", async () => {
  saveSettings({ theme: "light", palette: "purple", outputFolder: "" });
  mockIPC((command) => {
    if (command === "plugin:dialog|open") return "C:\\backup.json";
    if (command === "import_settings_backup")
      return {
        version: 1,
        theme: "dark",
        palette: "blue",
        outputFolder: "C:\\Pasta antiga",
      };
    if (command === "validate_output_folder") throw { code: "not_found" };
    throw new Error(command);
  });
  render(<SettingsBackup />);
  fireEvent.click(screen.getByRole("button", { name: "Importar backup" }));
  await screen.findByRole("button", { name: "Restaurar preferências" });
  expect(readSettings().theme).toBe("light");
  fireEvent.click(
    screen.getByRole("button", { name: "Restaurar preferências" }),
  );
  await screen.findByText(
    /Preferências restauradas. A pasta salva não está disponível/,
  );
  expect(readSettings()).toMatchObject({
    theme: "dark",
    palette: "blue",
    outputFolder: "",
  });
});

test("backup inválido preserva as configurações atuais", async () => {
  saveSettings({ theme: "light", palette: "pink" });
  mockIPC((command) => {
    if (command === "plugin:dialog|open") return "C:\\invalido.json";
    throw { code: "invalid_settings_backup" };
  });
  render(<SettingsBackup />);
  fireEvent.click(screen.getByRole("button", { name: "Importar backup" }));
  expect((await screen.findByRole("alert")).textContent).toContain(
    "backup compatível",
  );
  expect(readSettings()).toMatchObject({ theme: "light", palette: "pink" });
  expect(
    screen.queryByRole("button", { name: "Restaurar preferências" }),
  ).toBeNull();
});
