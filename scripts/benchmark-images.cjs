const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");

const [mode, directory, sharpModule] = process.argv.slice(2);
const sharp = require(path.resolve(sharpModule));
const root = path.resolve(directory);
const cases = [
  ["oriented.jpg", 720, 480, 3],
  ["detailed.jpg", 4000, 3000, 3],
  ["alpha.png", 640, 480, 4],
  ["opaque.png", 640, 480, 3],
  ["odd.webp", 401, 307, 4],
];

async function prepare() {
  await fs.mkdir(root, { recursive: true });
  for (const [name, width, height, channels] of cases) {
    const pixels = Buffer.alloc(width * height * channels);
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const offset = (y * width + x) * channels;
        const detail = Math.imul(x ^ (y << 12), 0x45d9f3b) >>> 24;
        pixels[offset] = (x / 8 + detail / 5) % 256;
        pixels[offset + 1] = (y / 6 + detail / 4) % 256;
        pixels[offset + 2] = ((x + y) / 10 + detail / 6) % 256;
        if (channels === 4) pixels[offset + 3] = Math.min(255, Math.floor(x / width * 340));
      }
    }
    let image = sharp(pixels, { raw: { width, height, channels } });
    if (name === "oriented.jpg") image = image.withMetadata({ orientation: 6 });
    if (name.endsWith(".jpg")) image = image.jpeg({ quality: 98 });
    if (name.endsWith(".webp")) image = image.webp({ lossless: true });
    await image.toFile(path.join(root, name));
  }
  await fs.writeFile(path.join(root, "corrupt.png"), Buffer.from("89504e470d0a1a0a", "hex"));
  await fs.copyFile(path.join(root, "alpha.png"), path.join(root, "misleading.jpg"));
}

async function compare() {
  const rows = JSON.parse(await fs.readFile(path.join(root, "rust.json"), "utf8"));
  const report = [];
  for (const row of rows) {
    if (row.name === "corrupt.png" || row.name === "misleading.jpg") {
      assert.equal(row.result.status, "failed");
      continue;
    }
    assert.equal(row.result.status, "saved");
    const input = path.join(root, row.name);
    const rustFile = path.join(root, "rust", row.result.fileName);
    const started = performance.now();
    let reference = sharp(input).autoOrient().resize({
      width: 1600, height: 1200, fit: "inside", withoutEnlargement: true,
    });
    if (row.name.endsWith(".jpg")) reference = reference.jpeg({ quality: 80 });
    if (row.name.endsWith(".png")) reference = reference.png({ compressionLevel: 9, adaptiveFiltering: true });
    if (row.name.endsWith(".webp")) reference = reference.webp({ lossless: true });
    const referenceBytes = await reference.toBuffer();
    const referenceMs = performance.now() - started;
    await fs.writeFile(path.join(root, "sharp-" + row.name), referenceBytes);
    const rust = await sharp(rustFile).metadata();
    const expected = await sharp(referenceBytes).metadata();
    assert.deepEqual([rust.format, rust.width, rust.height, rust.hasAlpha],
      [expected.format, expected.width, expected.height, expected.hasAlpha]);
    const actualPixels = await sharp(rustFile).ensureAlpha().raw().toBuffer();
    const expectedPixels = await sharp(referenceBytes).ensureAlpha().raw().toBuffer();
    let squaredError = 0;
    let channels = 0;
    for (let offset = 0; offset < actualPixels.length; offset += 4) {
      assert.equal(actualPixels[offset + 3], expectedPixels[offset + 3]);
      if (actualPixels[offset + 3] === 0) continue;
      for (let channel = 0; channel < 3; channel++) {
        squaredError += (actualPixels[offset + channel] - expectedPixels[offset + channel]) ** 2;
        channels++;
      }
    }
    if (!row.name.endsWith(".jpg")) assert.equal(squaredError, 0);
    if (row.name.endsWith(".jpg")) assert.ok(row.result.outputBytes < row.result.inputBytes);
    report.push({
      sample: row.name, size: `${rust.width}×${rust.height}`, input: row.result.inputBytes,
      rustBytes: row.result.outputBytes, sharpBytes: referenceBytes.length,
      rustMs: Math.round(row.ms), sharpMs: Math.round(referenceMs),
      psnrBetween: squaredError === 0 ? "identical" : (10 * Math.log10(255 ** 2 * channels / squaredError)).toFixed(1),
    });
  }
  console.log("Sharp", sharp.versions.sharp, "libvips", sharp.versions.vips);
  console.table(report);
}

(mode === "prepare" ? prepare() : compare()).catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
