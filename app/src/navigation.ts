import {
  AudioLines,
  Download,
  Film,
  House,
  Image,
  Settings,
} from "lucide-react";

export const pages = [
  {
    path: "#/",
    label: "Início",
    icon: House,
    description: "Escolha uma ferramenta para trabalhar com seus arquivos.",
  },
  {
    path: "#/imagens",
    label: "Imagens",
    icon: Image,
    description: "Comprima, converta ou redimensione imagens.",
  },
  {
    path: "#/video",
    label: "Vídeo",
    icon: Film,
    description: "Reduza o tamanho de vídeos ou converta para outro formato.",
  },
  {
    path: "#/audio",
    label: "Áudio",
    icon: AudioLines,
    description: "Converta arquivos de áudio ou extraia o som de um vídeo.",
  },
  {
    path: "#/downloader",
    label: "Downloader",
    icon: Download,
    description: "Baixe mídias que você tem autorização para baixar.",
  },
  {
    path: "#/configuracoes",
    label: "Configurações",
    icon: Settings,
    description: "Ajuste a aparência e consulte informações do aplicativo.",
  },
] as const;

export type Page = (typeof pages)[number];

export function subscribeToRoute(onChange: () => void) {
  window.addEventListener("hashchange", onChange);
  return () => window.removeEventListener("hashchange", onChange);
}

export function getRoute() {
  return window.location.hash || "#/";
}
