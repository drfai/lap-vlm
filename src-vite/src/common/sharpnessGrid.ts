export function computeSharpnessGrid(data: Uint8ClampedArray, width: number, height: number) {
  const columns = Math.max(1, Math.round(16 * width / Math.max(width, height)));
  const rows = Math.max(1, Math.round(16 * height / Math.max(width, height)));
  const gray = new Float32Array(width * height);
  const smooth = new Float32Array(gray.length);
  for (let i = 0; i < gray.length; i++) {
    gray[i] = (0.2126 * data[i * 4] + 0.7152 * data[i * 4 + 1] + 0.0722 * data[i * 4 + 2]) * data[i * 4 + 3] / 255;
  }
  // A small Gaussian filter reduces noise before measuring Sobel gradient energy.
  for (let y = 1; y < height - 1; y++) {
    for (let x = 1; x < width - 1; x++) {
      const i = y * width + x;
      smooth[i] = (gray[i - width - 1] + 2 * gray[i - width] + gray[i - width + 1]
        + 2 * gray[i - 1] + 4 * gray[i] + 2 * gray[i + 1]
        + gray[i + width - 1] + 2 * gray[i + width] + gray[i + width + 1]) / 16;
    }
  }
  const energy = new Float32Array(columns * rows);
  const count = new Uint32Array(energy.length);
  const detail = new Uint32Array(energy.length);
  for (let y = 2; y < height - 2; y++) {
    for (let x = 2; x < width - 2; x++) {
      const i = y * width + x;
      if (data[i * 4 + 3] < 255) continue;
      const gx = (smooth[i - width + 1] + 2 * smooth[i + 1] + smooth[i + width + 1]
        - smooth[i - width - 1] - 2 * smooth[i - 1] - smooth[i + width - 1]) / 8;
      const gy = (smooth[i + width - 1] + 2 * smooth[i + width] + smooth[i + width + 1]
        - smooth[i - width - 1] - 2 * smooth[i - width] - smooth[i - width + 1]) / 8;
      const value = gx * gx + gy * gy;
      const cell = Math.floor(y * rows / height) * columns + Math.floor(x * columns / width);
      energy[cell] += value;
      count[cell]++;
      if (value >= 400) detail[cell]++;
    }
  }
  let peak = 0;
  for (let i = 0; i < energy.length; i++) {
    energy[i] = count[i] ? energy[i] / count[i] : 0;
    peak = Math.max(peak, energy[i]);
  }
  // Keep an absolute floor: a flat image must not get highlights just by ranking cells.
  const threshold = 64;
  const cells: { x: number; y: number; intensity: number }[] = [];
  for (let i = 0; i < energy.length; i++) {
    if (energy[i] >= threshold && detail[i] >= count[i] * 0.03 && count[i] > 0) {
      // Absolute energy dominates; within-image contrast only gently adjusts intensity.
      const intensity = (energy[i] - threshold) / (energy[i] + 100)
        * (0.75 + 0.25 * energy[i] / peak);
      cells.push({ x: i % columns, y: Math.floor(i / columns), intensity });
    }
  }
  return { columns, rows, cells };
}
