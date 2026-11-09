// @vitest-environment jsdom
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { DownloaderPanel } from "./DownloaderPanel";

afterEach(() => {
  cleanup();
  clearMocks();
});

test("mostra capa e quantidade, e baixa a playlist com o nome base escolhido", async () => {
  const url = "https://example.com/playlist";
  const folder = "C:\\Saída";
  mockIPC((command, args) => {
    switch (command) {
      case "inspect_download":
        expect(args).toEqual({ url });
        return {
          title: "Lista",
          duration: null,
          heights: [],
          has_thumbnail: false,
          subtitle_languages: [],
          playlist_size: 23,
          cover_url: "https://example.com/cover.jpg",
        };
      case "plugin:dialog|open":
        return folder;
      case "download_media":
        expect(args).toMatchObject({
          url,
          folder,
          kind: "audio",
          options: { namePrefix: "Jazz Wall" },
        });
        expect(args).not.toHaveProperty("authorized");
        expect(args).not.toHaveProperty("options.confirmedCount");
        return {
          files: [{ fileName: "Jazz Wall 1.mp3", bytes: 123 }],
          failed: 0,
          cancelled: 1,
        };
      default:
        throw new Error(`Comando inesperado: ${command}`);
    }
  });

  render(<DownloaderPanel />);
  fireEvent.change(screen.getByRole("textbox", { name: "Link da mídia" }), {
    target: { value: url },
  });
  fireEvent.click(screen.getByRole("button", { name: "Consultar link" }));
  await screen.findByText("Playlist · 23 itens");
  expect(
    document.querySelector(".download-cover img")?.getAttribute("src"),
  ).toBe("https://example.com/cover.jpg");
  const download = screen.getByRole<HTMLButtonElement>("button", {
    name: "Baixar playlist",
  });
  expect(screen.queryByRole("button", { name: "Escolher pasta" })).toBeNull();
  fireEvent.change(screen.getByRole("combobox", { name: "Saída" }), {
    target: { value: "audio" },
  });
  fireEvent.change(
    screen.getByRole("textbox", { name: "Nome base (opcional)" }),
    { target: { value: "Jazz Wall" } },
  );
  expect(screen.getByText("Jazz Wall 1.mp3, Jazz Wall 2.mp3…")).toBeTruthy();
  expect(screen.queryByRole("checkbox")).toBeNull();
  expect(download.disabled).toBe(false);
  fireEvent.click(download);
  expect(await screen.findByText("1 de 23 itens baixados")).toBeTruthy();
  expect(screen.getByRole("button", { name: "Abrir pasta" })).toBeTruthy();
});
