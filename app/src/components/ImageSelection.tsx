import { useCallback, useEffect, useRef, useState } from "react";
import { ImageIcon, ImagePlus, X } from "lucide-react";
import { isValidPercent, PercentField } from "@/components/PercentField";
import { Button } from "@/components/ui/button";
import { readSettings } from "@/lib/settings";
import {
  displaySize,
  mediaErrorMessage,
  type CompressionResult,
  type ImageFormat,
  type ImageJobUpdate,
  type ImageOptions,
  type ImagePreview,
  type MediaInspection,
  type PlannedOutput,
} from "@/lib/media-contract";
import {
  cancelImageJob,
  chooseImages,
  chooseOutputFolder,
  compressImages,
  inspectImages,
  listenForImageDrops,
  openImageOutputFolder,
  previewImageConversion,
  previewImageOutputs,
  validateOutputFolder,
} from "@/lib/media-selection";

function itemPath(item: MediaInspection) {
  return item.status === "selected" ? item.media.path : item.path;
}

function isValidEdge(value: string) {
  return (
    value === "" ||
    (/^\d+$/.test(value) && Number(value) >= 1 && Number(value) <= 16_384)
  );
}

const sections = [
  { value: "format", label: "Formato" },
  { value: "resize", label: "Redimensionar" },
  { value: "preview", label: "Prévia" },
] as const;

export function ImageSelection() {
  const [items, setItems] = useState<MediaInspection[]>([]);
  const [folder, setFolder] = useState(() => readSettings().outputFolder);
  const [outputPlan, setOutputPlan] = useState<PlannedOutput[]>([]);
  const [batch, setBatch] = useState<{
    folder: string;
    results: CompressionResult[];
  } | null>(null);
  const [job, setJob] = useState<ImageJobUpdate | null>(null);
  const [cancelRequested, setCancelRequested] = useState(false);
  const [width, setWidth] = useState("");
  const [height, setHeight] = useState("");
  const [resizeMode, setResizeMode] = useState<"fit" | "percent" | "exact">(
    "fit",
  );
  const [percent, setPercent] = useState("50");
  const [section, setSection] =
    useState<(typeof sections)[number]["value"]>("format");
  const [previewPath, setPreviewPath] = useState("");
  const [preview, setPreview] = useState<{
    key: string;
    result: ImagePreview;
  } | null>(null);
  const [format, setFormat] = useState<ImageFormat | "">("");
  const [jpegQuality, setJpegQuality] = useState(80);
  const [error, setError] = useState("");
  const [busyMessage, setBusyMessage] = useState("");
  const busy = busyMessage !== "";
  const inProgress = useRef(false);
  const comparisonDialog = useRef<HTMLDialogElement>(null);
  const hasResize = resizeMode !== "fit" || width !== "" || height !== "";
  const hasJpegOutput =
    format === "jpeg" ||
    (!format &&
      items.some(
        (item) => item.status === "selected" && item.media.format === "jpeg",
      ));
  const invalidWidth = !isValidEdge(width);
  const invalidHeight = !isValidEdge(height);
  const invalidPercent = !isValidPercent(percent);
  const showScaledDimensions = resizeMode === "percent" && !invalidPercent;
  const invalidResolution =
    resizeMode === "exact" &&
    (!width || !height || Number(width) * Number(height) > 40_000_000);
  const invalidResize =
    resizeMode === "percent"
      ? invalidPercent
      : invalidWidth || invalidHeight || invalidResolution;
  const selectedPaths = items.flatMap((item) =>
    item.status === "selected" ? [item.media.path] : [],
  );
  const options: ImageOptions = {
    ...(hasResize && {
      resize:
        resizeMode === "percent"
          ? { percent: Number(percent) }
          : {
              width: width ? Number(width) : undefined,
              height: height ? Number(height) : undefined,
              ...(resizeMode === "exact" && { exact: true }),
            },
    }),
    ...(format && { format }),
    ...(hasJpegOutput && { jpegQuality }),
  };
  const samplePath = selectedPaths.includes(previewPath)
    ? previewPath
    : selectedPaths[0];
  const previewKey = JSON.stringify([samplePath, options]);
  const sample =
    !invalidResize && preview?.key === previewKey ? preview.result : null;
  const saved = batch?.results.filter((item) => item.status === "saved") ?? [];
  const failed =
    batch?.results.filter((item) => item.status === "failed").length ?? 0;
  const cancelled = (batch?.results.length ?? 0) - saved.length - failed;
  const totalDifference = saved.reduce(
    (total, item) => total + item.inputBytes - item.outputBytes,
    0,
  );
  const sizeSummary =
    totalDifference === 0
      ? "Tamanho total mantido"
      : `${totalDifference > 0 ? "Economia" : "Aumento"} de ${Math.abs(totalDifference).toLocaleString("pt-BR")} bytes`;

  const runSelection = useCallback(
    async (
      select: () => Promise<void>,
      message = "Aguardando seleção ou validação…",
    ) => {
      if (inProgress.current) return;
      inProgress.current = true;
      setBusyMessage(message);
      setError("");
      try {
        await select();
      } catch (failure) {
        setError(mediaErrorMessage(failure));
      } finally {
        inProgress.current = false;
        setBusyMessage("");
      }
    },
    [],
  );

  const addImages = useCallback(async (paths: string[]) => {
    if (paths.length === 0) return;
    const results = await inspectImages([...new Set(paths)]);
    setPreview(null);
    setOutputPlan([]);
    setItems((current) => {
      const merged = new Map(current.map((item) => [itemPath(item), item]));
      for (const item of results) merged.set(itemPath(item), item);
      return [...merged.values()];
    });
  }, []);

  useEffect(() => {
    let active = true;
    const subscription = listenForImageDrops((paths) => {
      if (active) void runSelection(() => addImages(paths));
    }).catch(() => {
      if (active) setError("Não foi possível ativar a seleção por arraste.");
    });
    return () => {
      active = false;
      void subscription.then((unlisten) => unlisten?.());
    };
  }, [addImages, runSelection]);

  async function selectImages() {
    await runSelection(async () => {
      const selected = await chooseImages();
      if (selected) await addImages(selected);
    });
  }

  async function selectFolder() {
    await runSelection(async () => {
      const selected = await chooseOutputFolder();
      if (selected) {
        setFolder(await validateOutputFolder(selected));
        setOutputPlan([]);
      }
    });
  }

  async function previewOutputs() {
    await runSelection(async () => {
      setOutputPlan([]);
      setOutputPlan(
        await previewImageOutputs(selectedPaths, folder, format || undefined),
      );
    });
  }

  async function processImages() {
    if (invalidResize) return;
    await runSelection(async () => {
      setBatch(null);
      setOutputPlan([]);
      setCancelRequested(false);
      let active = true;
      try {
        const results = await compressImages(
          selectedPaths,
          folder,
          options,
          (update) => {
            if (active)
              setJob((current) =>
                !current || update.completed >= current.completed
                  ? update
                  : current,
              );
          },
        );
        setBatch({ folder, results });
      } finally {
        active = false;
        setJob(null);
      }
    }, "Processando imagens…");
  }

  async function generatePreview() {
    if (!samplePath || invalidResize) return;
    await runSelection(async () => {
      setPreview(null);
      const result = await previewImageConversion(samplePath, options);
      setPreview({ key: previewKey, result });
    }, "Gerando prévia…");
  }

  async function cancelJob() {
    if (!job || cancelRequested) return;
    if (
      job.total > 1 &&
      !window.confirm(
        "Cancelar o lote? Os arquivos já gravados serão mantidos.",
      )
    )
      return;
    setCancelRequested(true);
    try {
      await cancelImageJob(job.id);
    } catch (failure) {
      setCancelRequested(false);
      setError(mediaErrorMessage(failure));
    }
  }

  return (
    <div className="image-workspace">
      <section
        className="surface media-selection"
        aria-labelledby="image-selection-title"
      >
        <div className="image-drop-area">
          <ImageIcon size={28} aria-hidden="true" />
          <h2 id="image-selection-title">Arquivos de entrada</h2>
          <Button
            type="button"
            className="h-8 px-3 text-xs"
            onClick={selectImages}
            disabled={busy}
          >
            <ImagePlus size={14} aria-hidden="true" /> Selecionar imagens
          </Button>
          <p>Ou arraste aqui · JPEG, PNG e WebP</p>
        </div>
        <p role="status">{busyMessage}</p>
        {job && (
          <div className="media-actions mt-2">
            <div>
              <p aria-live="polite">
                {cancelRequested || job.status === "cancelling"
                  ? "Cancelando…"
                  : job.status === "queued"
                    ? "Na fila"
                    : `${job.completed} de ${job.total} arquivos concluídos`}
              </p>
              <progress
                aria-label="Progresso do lote"
                max={job.total}
                value={job.completed}
              />
            </div>
            {(job.status === "queued" ||
              job.status === "running" ||
              job.status === "cancelling") && (
              <button
                type="button"
                className="media-text-button"
                disabled={cancelRequested || job.status === "cancelling"}
                onClick={cancelJob}
              >
                Cancelar lote
              </button>
            )}
          </div>
        )}
        {items.length === 0 ? (
          <p className="media-empty">Nenhum arquivo selecionado</p>
        ) : (
          <>
            <ul className="media-list" aria-label="Imagens selecionadas">
              {items.map((item) => {
                const path = itemPath(item);
                const name =
                  item.status === "selected"
                    ? item.media.name
                    : path.split(/[\\/]/).pop() || path;
                return (
                  <li key={path}>
                    <div className="image-file">
                      {item.status === "selected" &&
                        (item.media.thumbnail ? (
                          <img
                            className="image-thumbnail"
                            src={item.media.thumbnail}
                            alt=""
                          />
                        ) : (
                          <span className="image-thumbnail">
                            <ImageIcon size={18} aria-hidden="true" />
                          </span>
                        ))}
                      <div>
                        <strong title={path}>{name}</strong>
                        {item.status === "selected" ? (
                          <p>
                            {item.media.format.toUpperCase()} ·{" "}
                            {displaySize(item.media.sizeBytes)}
                            {showScaledDimensions &&
                              item.media.dimensions &&
                              ` · ${item.media.dimensions.map((edge) => Math.max(1, Math.round((edge * Number(percent)) / 100))).join(" × ")} px previstos`}
                          </p>
                        ) : (
                          <p className="media-error">
                            {mediaErrorMessage(item.error)}
                          </p>
                        )}
                      </div>
                    </div>
                    <button
                      type="button"
                      className="media-text-button"
                      aria-label={`Remover ${name}`}
                      disabled={busy}
                      onClick={() => {
                        setItems((current) =>
                          current.filter((entry) => itemPath(entry) !== path),
                        );
                        setOutputPlan([]);
                      }}
                    >
                      <X size={15} aria-hidden="true" />
                    </button>
                  </li>
                );
              })}
            </ul>
            <button
              type="button"
              className="media-text-button"
              onClick={() => {
                setItems([]);
                setOutputPlan([]);
              }}
              disabled={busy}
            >
              Limpar seleção
            </button>
          </>
        )}
      </section>
      <section
        className="surface media-selection"
        aria-labelledby="image-settings-title"
      >
        <h2 id="image-settings-title">Ajustes da imagem</h2>
        <div
          className="image-sections"
          role="group"
          aria-label="Ajustes da imagem"
        >
          {sections.map(({ value, label }) => (
            <button
              type="button"
              key={value}
              aria-pressed={section === value}
              onClick={() => setSection(value)}
            >
              {label}
            </button>
          ))}
        </div>
        {section === "format" && (
          <fieldset className="mt-3" disabled={busy}>
            <legend className="text-xs font-semibold">
              Formato e qualidade
            </legend>
            <div className="resize-fields">
              <label>
                Formato de saída
                <select
                  value={format}
                  onChange={(event) => {
                    setFormat(event.target.value as ImageFormat | "");
                    setOutputPlan([]);
                  }}
                >
                  <option value="">Manter original</option>
                  <option value="jpeg">JPEG</option>
                  <option value="png">PNG</option>
                  <option value="webp">WebP</option>
                </select>
              </label>
              {hasJpegOutput && (
                <label>
                  Qualidade JPEG <span aria-hidden="true">{jpegQuality}</span>
                  <input
                    type="range"
                    min="1"
                    max="100"
                    value={jpegQuality}
                    onChange={(event) =>
                      setJpegQuality(Number(event.target.value))
                    }
                  />
                </label>
              )}
            </div>
            {hasJpegOutput && (
              <p>O JPEG substitui áreas transparentes por fundo branco.</p>
            )}
            {format === "png" && (
              <p>O PNG usa compressão sem perdas e mantém a transparência.</p>
            )}
            {format === "webp" && (
              <p>O WebP usa compressão sem perdas em 8 bits por canal.</p>
            )}
          </fieldset>
        )}
        {section === "resize" && (
          <fieldset className="mt-3" disabled={busy}>
            <legend className="text-xs font-semibold">
              Modo de redimensionamento
            </legend>
            <div className="resize-fields">
              <label>
                Modo
                <select
                  value={resizeMode}
                  onChange={(event) =>
                    setResizeMode(event.target.value as typeof resizeMode)
                  }
                >
                  <option value="fit">Limites em pixels</option>
                  <option value="percent">Porcentagem</option>
                  <option value="exact">Resolução exata</option>
                </select>
              </label>
            </div>
            {resizeMode === "percent" ? (
              <PercentField
                label="Escala"
                value={percent}
                onChange={setPercent}
              />
            ) : (
              <>
                <p>
                  {resizeMode === "exact"
                    ? "Usa as duas dimensões informadas. Se a proporção for diferente, a imagem será esticada."
                    : "Informe um ou dois limites para reduzir a imagem mantendo a proporção."}
                </p>
                <div className="resize-fields">
                  <label>
                    {resizeMode === "exact"
                      ? "Largura (px)"
                      : "Largura máxima (px)"}
                    <input
                      type="text"
                      inputMode="numeric"
                      value={width}
                      onChange={(event) => setWidth(event.target.value)}
                      aria-invalid={invalidWidth || undefined}
                      aria-describedby={
                        invalidWidth ? "resize-error" : undefined
                      }
                    />
                  </label>
                  <label>
                    {resizeMode === "exact"
                      ? "Altura (px)"
                      : "Altura máxima (px)"}
                    <input
                      type="text"
                      inputMode="numeric"
                      value={height}
                      onChange={(event) => setHeight(event.target.value)}
                      aria-invalid={invalidHeight || undefined}
                      aria-describedby={
                        invalidHeight ? "resize-error" : undefined
                      }
                    />
                  </label>
                </div>
              </>
            )}
            {invalidResize && (
              <p id="resize-error" className="media-error" role="alert">
                {resizeMode === "percent"
                  ? "Use uma porcentagem inteira de 1 a 100."
                  : resizeMode === "exact"
                    ? "Informe largura e altura de 1 a 16.384 pixels, com até 40 megapixels no total."
                    : "Use números inteiros de 1 a 16.384 pixels."}
              </p>
            )}
          </fieldset>
        )}
        {section === "preview" && (
          <div className="image-preview">
            {samplePath ? (
              <div className="resize-fields media-actions">
                <label>
                  Imagem da prévia
                  <select
                    value={samplePath}
                    disabled={busy}
                    onChange={(event) => setPreviewPath(event.target.value)}
                  >
                    {items.map(
                      (item) =>
                        item.status === "selected" && (
                          <option key={item.media.path} value={item.media.path}>
                            {item.media.name}
                          </option>
                        ),
                    )}
                  </select>
                </label>
                <Button
                  type="button"
                  className="h-8 px-3 text-xs"
                  disabled={busy || invalidResize}
                  onClick={generatePreview}
                >
                  Gerar prévia
                </Button>
              </div>
            ) : (
              <p>Adicione uma imagem para comparar antes e depois.</p>
            )}
            {invalidResize && <p>Confira os valores na aba Redimensionar.</p>}
            {sample && (
              <>
                <div className="image-comparison">
                  <figure>
                    <img src={sample.original} alt="Imagem original" />
                    <figcaption>
                      Original · {displaySize(sample.inputBytes)}
                    </figcaption>
                  </figure>
                  <figure>
                    <img src={sample.output} alt="Imagem processada" />
                    <figcaption>
                      Saída · {displaySize(sample.outputBytes)}
                    </figcaption>
                  </figure>
                </div>
                <button
                  type="button"
                  className="media-text-button"
                  onClick={() => comparisonDialog.current?.showModal()}
                >
                  Ampliar comparação
                </button>
                <dialog
                  className="image-zoom"
                  ref={comparisonDialog}
                  aria-labelledby="image-zoom-title"
                >
                  <div className="media-actions">
                    <h2 id="image-zoom-title">Comparar qualidade</h2>
                    <form method="dialog">
                      <button type="submit" className="media-text-button">
                        Fechar
                      </button>
                    </form>
                  </div>
                  <div className="image-comparison">
                    <figure>
                      <img
                        src={sample.original}
                        alt="Imagem original ampliada"
                      />
                      <figcaption>Original</figcaption>
                    </figure>
                    <figure>
                      <img
                        src={sample.output}
                        alt="Imagem processada ampliada"
                      />
                      <figcaption>Saída</figcaption>
                    </figure>
                  </div>
                </dialog>
                <p>
                  {sample.width} × {sample.height} px ·{" "}
                  {sample.inputBytes === sample.outputBytes
                    ? "Tamanho mantido"
                    : `${sample.outputBytes < sample.inputBytes ? "Redução" : "Aumento"} de ${displaySize(Math.abs(sample.outputBytes - sample.inputBytes))}`}
                </p>
              </>
            )}
          </div>
        )}
        <div className="image-destination">
          <div className="media-actions">
            <h2>Pasta de saída</h2>
            <button
              type="button"
              className="media-text-button"
              onClick={selectFolder}
              disabled={busy}
            >
              Escolher pasta
            </button>
          </div>
          <p className="media-folder" title={folder}>
            {folder || "Nenhuma pasta selecionada"}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            className="mt-3 h-8 px-3 text-xs"
            onClick={previewOutputs}
            disabled={busy || !folder || selectedPaths.length === 0}
          >
            Pré-visualizar nomes
          </Button>
          <Button
            type="button"
            className="mt-3 h-8 px-3 text-xs"
            onClick={processImages}
            disabled={
              busy || !folder || selectedPaths.length === 0 || invalidResize
            }
          >
            {format
              ? "Converter imagens"
              : hasResize
                ? "Redimensionar imagens"
                : "Comprimir imagens"}
          </Button>
        </div>
        {outputPlan.length > 0 && (
          <ul className="media-list" aria-label="Nomes previstos de saída">
            {outputPlan.map(({ inputPath, fileName }) => (
              <li key={inputPath}>
                <strong title={inputPath}>{fileName}</strong>
              </li>
            ))}
          </ul>
        )}
      </section>
      {batch && batch.results.length > 0 && (
        <section
          className="surface image-results"
          aria-labelledby="compression-results"
        >
          <div className="media-actions">
            <h2 id="compression-results">Último lote</h2>
            {saved.length > 0 && (
              <Button
                type="button"
                className="h-8 px-3 text-xs"
                disabled={busy}
                onClick={() =>
                  runSelection(
                    () => openImageOutputFolder(batch.folder),
                    "Abrindo pasta de saída…",
                  )
                }
              >
                Abrir pasta de saída
              </Button>
            )}
          </div>
          <p aria-live="polite">
            {saved.length}{" "}
            {saved.length === 1 ? "arquivo gravado" : "arquivos gravados"}
            {" · "}
            {failed} {failed === 1 ? "falha" : "falhas"}
            {cancelled > 0 &&
              ` · ${cancelled} ${cancelled === 1 ? "cancelado" : "cancelados"}`}
            {saved.length > 0 && ` · ${sizeSummary}`}
          </p>
          {saved.length > 0 && <p className="media-folder">{batch.folder}</p>}
          <ul className="media-list">
            {batch.results.map((result) => (
              <li key={result.inputPath}>
                <div>
                  {result.status === "saved" ? (
                    <>
                      <strong title={result.inputPath}>
                        {result.fileName}
                      </strong>
                      <p>
                        {result.width} × {result.height} px
                      </p>
                      <p>
                        {result.inputBytes.toLocaleString("pt-BR")} →{" "}
                        {result.outputBytes.toLocaleString("pt-BR")} bytes
                      </p>
                    </>
                  ) : (
                    <>
                      <strong title={result.inputPath}>
                        {result.inputPath.split(/[\\/]/).pop()}
                      </strong>
                      <p className="media-error">
                        {result.status === "cancelled"
                          ? "Processamento cancelado."
                          : mediaErrorMessage(result.error)}
                      </p>
                    </>
                  )}
                </div>
              </li>
            ))}
          </ul>
        </section>
      )}
      {error && (
        <p className="media-error" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
