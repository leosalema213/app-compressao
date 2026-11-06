// @vitest-environment jsdom
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import App from "./App";

afterEach(() => {
  cleanup();
  clearMocks();
  window.history.replaceState(null, "", "#/");
  window.localStorage.clear();
});

test("mostra a versão devolvida pelo comando Rust", async () => {
  window.history.replaceState(null, "", "#/configuracoes");
  mockIPC((command) => {
    expect(command).toBe("get_app_diagnostic");
    return { version: "1.2.3" };
  });

  render(<App />);
  fireEvent.click(
    screen.getByRole("button", { name: "Consultar diagnóstico" }),
  );

  expect(await screen.findByText("Versão do aplicativo: 1.2.3")).toBeTruthy();
});

test("mostra um erro e permite tentar novamente se a chamada falhar", async () => {
  window.history.replaceState(null, "", "#/configuracoes");
  mockIPC(() => {
    throw new Error("Falha simulada");
  });

  render(<App />);
  const button = screen.getByRole("button", { name: "Consultar diagnóstico" });
  fireEvent.click(button);

  expect((await screen.findByRole("alert")).textContent).toContain(
    "Não foi possível consultar a versão.",
  );
  expect(button.hasAttribute("disabled")).toBe(false);
});

test("expõe as seis rotas na navegação e marca a página atual", async () => {
  render(<App />);
  const navigation = within(
    screen.getByRole("navigation", { name: "Navegação principal" }),
  );
  expect(navigation.getAllByRole("link")).toHaveLength(6);
  expect(
    navigation
      .getByRole("link", { name: "Início" })
      .getAttribute("aria-current"),
  ).toBe("page");

  for (const label of [
    "Imagens",
    "Vídeo",
    "Áudio",
    "Downloader",
    "Configurações",
    "Início",
  ]) {
    const link = navigation.getByRole("link", { name: label });
    fireEvent.click(link);
    await screen.findByRole("heading", { level: 1, name: label });
    expect(link.getAttribute("aria-current")).toBe("page");
  }
});

test("oferece uma volta ao início para endereços desconhecidos", async () => {
  window.history.replaceState(null, "", "#/inexistente");
  render(<App />);
  expect(
    screen.getByRole("heading", { name: "Página não encontrada" }),
  ).toBeTruthy();

  fireEvent.click(screen.getByRole("link", { name: "Voltar ao início" }));
  expect(
    await screen.findByRole("heading", { level: 1, name: "Início" }),
  ).toBeTruthy();
});

test("acompanha voltar e avançar no histórico", async () => {
  render(<App />);
  const navigation = within(screen.getByRole("navigation"));
  fireEvent.click(navigation.getByRole("link", { name: "Imagens" }));
  await screen.findByRole("heading", { level: 1, name: "Imagens" });

  window.history.back();
  await screen.findByRole("heading", { level: 1, name: "Início" });

  window.history.forward();
  await screen.findByRole("heading", { level: 1, name: "Imagens" });
});

test("aplica tema e cor de destaque e recupera as escolhas ao reabrir", () => {
  const { unmount } = render(<App />);
  fireEvent.change(screen.getByRole("combobox", { name: "Tema" }), {
    target: { value: "dark" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Azul" }));
  expect(document.documentElement.dataset.theme).toBe("dark");
  expect(document.documentElement.dataset.palette).toBe("blue");
  unmount();

  render(<App />);
  expect(
    screen.getByRole("button", { name: "Azul" }).getAttribute("aria-pressed"),
  ).toBe("true");
  expect(
    screen.getByRole<HTMLSelectElement>("combobox", { name: "Tema" }).value,
  ).toBe("dark");
});
