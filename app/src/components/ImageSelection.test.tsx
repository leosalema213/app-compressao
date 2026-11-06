// @vitest-environment jsdom
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import { listenForImageDrops } from "@/lib/media-selection";
import inspection from "../../tests/fixtures/image-inspection.json";
import { ImageSelection } from "./ImageSelection";

vi.mock(import("@/lib/media-selection"), async (importOriginal) => ({
  ...(await importOriginal()),
  listenForImageDrops: vi.fn(async () => undefined),
}));

afterEach(() => {
  cleanup();
  clearMocks();
  vi.clearAllMocks();
});

test("lista arquivos aceitos e rejeitados e permite remover ou limpar itens", async () => {
  mockIPC((command) => {
    if (command === "plugin:dialog|open") return ["foto.png", "outra.png"];
    expect(command).toBe("inspect_images");
    return inspection;
  });

  render(<ImageSelection />);
  fireEvent.click(screen.getByRole("button", { name: "Selecionar imagens" }));
  expect(await screen.findByText("PNG · 128 bytes")).toBeTruthy();
  expect(
    screen.getByText("O conteúdo do arquivo não corresponde à extensão."),
  ).toBeTruthy();

  fireEvent.click(screen.getByRole("button", { name: "Remover foto.png" }));
  expect(screen.queryByText("foto.png")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Limpar seleção" }));
  expect(screen.getByText("Nenhum arquivo selecionado")).toBeTruthy();
});

test("encaminha os caminhos arrastados à mesma inspeção e deduplica os itens", async () => {
  const inspect = vi.fn<(args: unknown) => typeof inspection>(() => inspection);
  mockIPC((command, args) => {
    expect(command).toBe("inspect_images");
    return inspect(args);
  });
  render(<ImageSelection />);
  const [onDrop] = vi.mocked(listenForImageDrops).mock.calls[0];

  await act(() => onDrop(["foto.png", "foto.png"]));
  await act(() => onDrop(["foto.png"]));
  expect(inspect).toHaveBeenNthCalledWith(1, { paths: ["foto.png"] });
  expect(inspect).toHaveBeenCalledTimes(2);
  expect(screen.getAllByText("foto.png")).toHaveLength(1);
});

test("cancelar o diálogo não chama a inspeção nem apresenta erro", async () => {
  const handle = vi.fn(() => null);
  mockIPC((command) => {
    expect(command).toBe("plugin:dialog|open");
    return handle();
  });
  render(<ImageSelection />);
  const button = screen.getByRole<HTMLButtonElement>("button", {
    name: "Selecionar imagens",
  });
  fireEvent.click(button);

  await waitFor(() => expect(button.disabled).toBe(false));
  expect(handle).toHaveBeenCalledTimes(1);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("apresenta a pasta após o Rust validar o caminho", async () => {
  const folder = "C:\\Saída de imagens";
  mockIPC((command, args) => {
    if (command === "plugin:dialog|open") return folder;
    expect(command).toBe("validate_output_folder");
    expect(args).toEqual({ path: folder });
    return folder;
  });
  render(<ImageSelection />);
  fireEvent.click(screen.getByRole("button", { name: "Escolher pasta" }));
  expect(await screen.findByText(folder)).toBeTruthy();
});

test("pré-visualiza nomes apenas para arquivos aceitos e descarta o plano ao limpar a seleção", async () => {
  const folder = "C:\\Saída";
  const inputPath = "C:\\Mídia\\foto.png";
  mockIPC((command, args) => {
    switch (command) {
      case "plugin:dialog|open": {
        const options = (args as { options: { directory?: boolean } }).options;
        return options.directory ? folder : [inputPath, "C:\\Mídia\\outra.png"];
      }
      case "inspect_images":
        return inspection;
      case "validate_output_folder":
        return folder;
      case "preview_image_outputs":
        expect(args).toEqual({ paths: [inputPath], folder });
        return [{ inputPath, fileName: "foto-2.png" }];
      default:
        throw new Error(`Comando inesperado: ${command}`);
    }
  });

  render(<ImageSelection />);
  const preview = screen.getByRole<HTMLButtonElement>("button", {
    name: "Pré-visualizar nomes",
  });
  expect(preview.disabled).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: "Selecionar imagens" }));
  await screen.findByText("foto.png");
  expect(preview.disabled).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: "Escolher pasta" }));
  await screen.findByText(folder);
  fireEvent.click(preview);
  expect(await screen.findByText("foto-2.png")).toBeTruthy();

  fireEvent.click(screen.getByRole("button", { name: "Limpar seleção" }));
  expect(screen.queryByText("foto-2.png")).toBeNull();
  expect(preview.disabled).toBe(true);
});

test("mostra tamanhos de saída e falhas no resultado do lote", async () => {
  const folder = "C:\\Saída";
  const otherFolder = "C:\\Outra saída";
  let selectedFolder = folder;
  const openOutput = vi.fn();
  const inputPath = "C:\\Mídia\\foto.png";
  const corruptPath = "C:\\Mídia\\corrupta.png";
  mockIPC((command, args) => {
    switch (command) {
      case "plugin:dialog|open": {
        const options = (args as { options: { directory?: boolean } }).options;
        return options.directory ? selectedFolder : [inputPath, corruptPath];
      }
      case "inspect_images":
        return [
          inspection[0],
          {
            status: "selected",
            media: {
              kind: "image",
              format: "png",
              path: corruptPath,
              name: "corrupta.png",
              sizeBytes: 8,
            },
          },
        ];
      case "validate_output_folder":
        return selectedFolder;
      case "compress_images":
        expect(args).toMatchObject({ paths: [inputPath, corruptPath], folder });
        return [
          {
            status: "saved",
            inputPath,
            fileName: "foto-2.png",
            inputBytes: 128,
            outputBytes: 96,
            width: 32,
            height: 24,
          },
          {
            status: "failed",
            inputPath: corruptPath,
            error: { code: "invalid_image" },
          },
        ];
      case "open_image_output_folder":
        openOutput(args);
        return undefined;
      default:
        throw new Error(`Comando inesperado: ${command}`);
    }
  });
  render(<ImageSelection />);
  const compress = screen.getByRole<HTMLButtonElement>("button", {
    name: "Comprimir imagens",
  });
  expect(compress.disabled).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: "Selecionar imagens" }));
  await screen.findByText("foto.png");
  fireEvent.click(screen.getByRole("button", { name: "Escolher pasta" }));
  await screen.findByText(folder);
  fireEvent.click(compress);
  expect(await screen.findByText("foto-2.png")).toBeTruthy();
  expect(screen.getByText("128 → 96 bytes")).toBeTruthy();
  expect(
    screen.getByText("1 arquivo gravado · 1 falha · Economia de 32 bytes"),
  ).toBeTruthy();
  expect(
    screen.getByText("Não foi possível decodificar a imagem."),
  ).toBeTruthy();

  selectedFolder = otherFolder;
  fireEvent.click(screen.getByRole("button", { name: "Escolher pasta" }));
  await screen.findByText(otherFolder);
  fireEvent.click(screen.getByRole("button", { name: "Abrir pasta de saída" }));
  await waitFor(() => expect(openOutput).toHaveBeenCalledWith({ folder }));
});

test("valida os limites e apresenta as dimensões devolvidas pelo Rust", async () => {
  const folder = "C:\\Saída";
  const inputPath = "C:\\Mídia\\foto.png";
  mockIPC((command, args) => {
    switch (command) {
      case "plugin:dialog|open": {
        const options = (args as { options: { directory?: boolean } }).options;
        return options.directory ? folder : [inputPath];
      }
      case "inspect_images":
        return [inspection[0]];
      case "validate_output_folder":
        return folder;
      case "compress_images":
        expect(args).toMatchObject({
          paths: [inputPath],
          folder,
          resize: { width: 120, height: 80 },
        });
        return [
          {
            status: "saved",
            inputPath,
            fileName: "foto-2.png",
            inputBytes: 128,
            outputBytes: 96,
            width: 120,
            height: 60,
          },
        ];
      default:
        throw new Error(`Comando inesperado: ${command}`);
    }
  });
  render(<ImageSelection />);
  fireEvent.click(screen.getByRole("button", { name: "Selecionar imagens" }));
  await screen.findByText("foto.png");
  fireEvent.click(screen.getByRole("button", { name: "Escolher pasta" }));
  await screen.findByText(folder);

  const width = screen.getByRole("textbox", { name: "Largura máxima (px)" });
  fireEvent.change(width, { target: { value: "1.5" } });
  const resize = screen.getByRole<HTMLButtonElement>("button", {
    name: "Redimensionar imagens",
  });
  expect(resize.disabled).toBe(true);
  expect(screen.getByRole("alert").textContent).toContain("números inteiros");

  fireEvent.change(width, { target: { value: "120" } });
  fireEvent.change(
    screen.getByRole("textbox", { name: "Altura máxima (px)" }),
    { target: { value: "80" } },
  );
  expect(resize.disabled).toBe(false);
  fireEvent.click(resize);
  expect(await screen.findByText("120 × 60 px")).toBeTruthy();
});
test("envia formato e qualidade JPEG e atualiza a prévia de nomes", async () => {
  const folder = "C:\\Saída";
  const inputPath = "C:\\Mídia\\foto.png";
  mockIPC((command, args) => {
    switch (command) {
      case "plugin:dialog|open": {
        const options = (args as { options: { directory?: boolean } }).options;
        return options.directory ? folder : [inputPath];
      }
      case "inspect_images":
        return [inspection[0]];
      case "validate_output_folder":
        return folder;
      case "preview_image_outputs":
        expect(args).toEqual({ paths: [inputPath], folder, format: "jpeg" });
        return [{ inputPath, fileName: "foto-2.jpg" }];
      case "compress_images":
        expect(args).toMatchObject({
          paths: [inputPath],
          folder,
          format: "jpeg",
          jpegQuality: 65,
        });
        return [
          {
            status: "saved",
            inputPath,
            fileName: "foto-2.jpg",
            inputBytes: 128,
            outputBytes: 96,
            width: 24,
            height: 16,
          },
        ];
      default:
        throw new Error(`Comando inesperado: ${command}`);
    }
  });
  render(<ImageSelection />);
  fireEvent.click(screen.getByRole("button", { name: "Selecionar imagens" }));
  await screen.findByText("foto.png");
  fireEvent.click(screen.getByRole("button", { name: "Escolher pasta" }));
  await screen.findByText(folder);
  const format = screen.getByRole("combobox", { name: "Formato de saída" });
  fireEvent.change(format, { target: { value: "jpeg" } });
  fireEvent.change(screen.getByRole("slider", { name: "Qualidade JPEG" }), {
    target: { value: "65" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Pré-visualizar nomes" }));
  expect(await screen.findByText("foto-2.jpg")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Converter imagens" }));
  expect(
    await screen.findByRole("region", { name: "Último lote" }),
  ).toBeTruthy();
  expect(screen.getByText("foto-2.jpg")).toBeTruthy();
  fireEvent.change(format, { target: { value: "webp" } });
  expect(screen.queryByRole("slider", { name: "Qualidade JPEG" })).toBeNull();
});
