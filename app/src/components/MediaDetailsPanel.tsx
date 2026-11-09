import { useState } from "react";
import { isValidPercent, PercentField } from "@/components/PercentField";
import { Button } from "@/components/ui/button";
import { readSettings } from "@/lib/settings";
import {
  displaySize,
  mediaErrorMessage,
  type AvResult,
  type ImageJobUpdate,
  type MediaDetails,
  type StreamKind,
  type VideoPreset,
} from "@/lib/media-contract";
import {
  cancelImageJob,
  chooseAvFile,
  chooseOutputFolder,
  inspectAvFile,
  transcodeAv,
  validateOutputFolder,
} from "@/lib/media-selection";

export function MediaDetailsPanel({ kind }: { kind: StreamKind }) {
  const [media, setMedia] = useState<MediaDetails | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [folder, setFolder] = useState(() => readSettings().outputFolder);
  const [preset, setPreset] = useState<VideoPreset>("balanced");
  const [method, setMethod] = useState<"profile" | "target">("profile");
  const [percent, setPercent] = useState("50");
  const [job, setJob] = useState<ImageJobUpdate | null>(null);
  const [cancelRequested, setCancelRequested] = useState(false);
  const [processing, setProcessing] = useState(false);
  const [result, setResult] = useState<AvResult | null>(null);
  const video = kind === "video";
  const useTarget = video && method === "target";
  const validTarget = isValidPercent(percent);
  const invalidTarget = useTarget && (!validTarget || !media?.durationSeconds);

  async function selectFile() {
    setBusy(true);
    setError("");
    try {
      const selected = await chooseAvFile(kind);
      if (selected) {
        setMedia(null);
        setResult(null);
        setMedia(await inspectAvFile(selected, kind));
      }
    } catch (failure) {
      setError(mediaErrorMessage(failure));
    } finally {
      setBusy(false);
    }
  }

  async function selectFolder() {
    setBusy(true);
    setError("");
    try {
      const selected = await chooseOutputFolder();
      if (selected) setFolder(await validateOutputFolder(selected));
    } catch (failure) {
      setError(mediaErrorMessage(failure));
    } finally {
      setBusy(false);
    }
  }

  async function processFile() {
    if (!media || !folder || invalidTarget) return;
    setBusy(true);
    setProcessing(true);
    setCancelRequested(false);
    setResult(null);
    setError("");
    let active = true;
    try {
      setResult(
        await transcodeAv(
          media.path,
          folder,
          kind,
          useTarget ? "balanced" : preset,
          (update) => {
            if (active) setJob(update);
          },
          useTarget ? Number(percent) : undefined,
        ),
      );
    } catch (failure) {
      setError(mediaErrorMessage(failure));
    } finally {
      active = false;
      setJob(null);
      setProcessing(false);
      setBusy(false);
    }
  }

  async function cancelJob() {
    if (!job || cancelRequested) return;
    setCancelRequested(true);
    try {
      await cancelImageJob(job.id);
    } catch (failure) {
      setCancelRequested(false);
      setError(mediaErrorMessage(failure));
    }
  }

  return (
    <section className="surface" aria-labelledby="media-details-title">
      <div className="media-actions">
        <h2 id="media-details-title">Informações do arquivo</h2>
        <Button
          type="button"
          className="h-8 px-3 text-xs"
          disabled={busy}
          onClick={selectFile}
        >
          {video ? "Selecionar vídeo" : "Selecionar áudio ou vídeo"}
        </Button>
      </div>
      <div role="status">
        {busy && !processing && <p>Aguardando seleção ou leitura…</p>}
        {media ? (
          <>
            <p className="media-folder" title={media.path}>
              <strong>{media.name}</strong>
            </p>
            <p>
              {media.sizeBytes.toLocaleString("pt-BR")} bytes ·{" "}
              {media.durationSeconds === null
                ? "Duração indisponível"
                : `${media.durationSeconds.toLocaleString("pt-BR", { maximumFractionDigits: 2 })} s`}
            </p>
            {media.videoCodec && (
              <p>
                Vídeo: {media.videoCodec}
                {media.width && media.height
                  ? ` · ${media.width} × ${media.height} px`
                  : ""}
              </p>
            )}
            <p>Áudio: {media.audioCodec ?? "sem faixa"}</p>
          </>
        ) : (
          !busy && <p>Nenhum arquivo selecionado.</p>
        )}
      </div>
      <div className="mt-4">
        <div className="media-actions">
          <h2>Pasta de saída</h2>
          <Button
            type="button"
            className="h-8 px-3 text-xs"
            disabled={busy}
            onClick={selectFolder}
          >
            Escolher pasta
          </Button>
        </div>
        <p className="media-folder">{folder || "Nenhuma pasta selecionada"}</p>
        <p>
          {video
            ? "Saída MP4 com vídeo H.264 e áudio AAC, quando houver."
            : "Saída MP3 a 128 kb/s, incluindo extração de áudio de vídeo."}
        </p>
        {video && (
          <fieldset disabled={busy} className="mt-3">
            <legend className="text-xs font-semibold">Compressão</legend>
            <div className="resize-fields">
              <label>
                Método
                <select
                  value={method}
                  onChange={(event) =>
                    setMethod(event.target.value as typeof method)
                  }
                >
                  <option value="profile">Por perfil</option>
                  <option value="target">Por tamanho-alvo</option>
                </select>
              </label>
              {!useTarget && (
                <label>
                  Perfil de compressão
                  <select
                    disabled={busy}
                    value={preset}
                    onChange={(event) =>
                      setPreset(event.target.value as VideoPreset)
                    }
                  >
                    <option value="fast">Rápido</option>
                    <option value="balanced">Equilibrado</option>
                    <option value="smaller">Menor arquivo</option>
                  </select>
                </label>
              )}
            </div>
            {useTarget && (
              <>
                <PercentField
                  label="Tamanho do original"
                  value={percent}
                  onChange={setPercent}
                />
                {media && validTarget && (
                  <p>
                    Tamanho-alvo aproximado:{" "}
                    <strong>
                      {displaySize(
                        Math.round((media.sizeBytes * Number(percent)) / 100),
                      )}
                    </strong>
                  </p>
                )}
                <p>
                  50% significa tentar gerar um arquivo com metade do tamanho
                  original.
                </p>
                {!validTarget && (
                  <p className="media-error">
                    Use uma porcentagem inteira de 1 a 100.
                  </p>
                )}
                {media && !media.durationSeconds && (
                  <p className="media-error">
                    O tamanho-alvo precisa de um vídeo com duração conhecida.
                  </p>
                )}
              </>
            )}
          </fieldset>
        )}
        <Button
          type="button"
          className="mt-3 h-8 px-3 text-xs"
          disabled={busy || !media || !folder || invalidTarget}
          onClick={processFile}
        >
          {video ? "Gerar MP4" : "Gerar MP3"}
        </Button>
        {processing && (
          <div className="media-actions mt-2">
            <div>
              <p aria-live="polite">
                {cancelRequested
                  ? "Cancelando…"
                  : job?.status === "queued"
                    ? "Na fila"
                    : "Processando…"}
              </p>
              <progress
                aria-label="Progresso do processamento"
                max={100}
                value={
                  job?.status === "running" && media?.durationSeconds
                    ? job.completed
                    : undefined
                }
              />
            </div>
            {job && (job.status === "queued" || job.status === "running") && (
              <button
                type="button"
                className="media-text-button"
                disabled={cancelRequested}
                onClick={cancelJob}
              >
                Cancelar
              </button>
            )}
          </div>
        )}
        {result && (
          <div role="status">
            <p>
              <strong>{result.fileName}</strong>
            </p>
            <p>
              {result.inputBytes.toLocaleString("pt-BR")} →{" "}
              {result.outputBytes.toLocaleString("pt-BR")} bytes
            </p>
          </div>
        )}
      </div>
      {error && (
        <p className="media-error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}
