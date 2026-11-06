// @vitest-environment jsdom
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { DownloaderPanel } from "./DownloaderPanel";

afterEach(() => {
  cleanup();
  clearMocks();
});

test("exige autorização e confirmação da quantidade antes de baixar uma playlist", async () => {
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
          playlist_size: 2,
        };
      case "plugin:dialog|open":
      case "validate_output_folder":
        return folder;
      case "download_media":
        expect(args).toMatchObject({
          url,
          folder,
          kind: "video",
          authorized: true,
          options: { confirmedCount: 2 },
        });
        return {
          files: [{ fileName: "video.mp4", bytes: 123 }],
          failed: 0,
          cancelled: 1,
        };
      default:
        throw new Error(`Comando inesperado: ${command}`);
    }
  });

  render(<DownloaderPanel />);
  fireEvent.change(screen.getByRole("textbox", { name: "Link" }), {
    target: { value: url },
  });
  fireEvent.click(screen.getByRole("button", { name: "Consultar link" }));
  await screen.findByText("Playlist · 2 itens");
  const download = screen.getByRole<HTMLButtonElement>("button", {
    name: "Baixar playlist",
  });
  expect(download.disabled).toBe(true);

  fireEvent.click(screen.getByRole("button", { name: "Escolher pasta" }));
  await screen.findByText(folder);
  fireEvent.click(
    screen.getByRole("checkbox", {
      name: "Tenho autorização para baixar esta mídia",
    }),
  );
  expect(download.disabled).toBe(true);
  fireEvent.click(
    screen.getByRole("checkbox", {
      name: "Confirmo baixar 2 itens para a pasta selecionada",
    }),
  );
  expect(download.disabled).toBe(false);
  fireEvent.click(download);
  expect(
    await screen.findByText("1 gravado · 0 falhas · 1 cancelado"),
  ).toBeTruthy();
  expect(screen.getByText("video.mp4")).toBeTruthy();
});
