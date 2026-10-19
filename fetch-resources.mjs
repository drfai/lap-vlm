/**
 * 下载 Lap 构建所需的 resources: 5 个 AI 模型 + ffmpeg sidecar。
 *
 * 来源: scripts/download_models.ps1 + scripts/download_ffmpeg_sidecar.ps1
 * 为什么要重写: PowerShell IWR 在这个网络下对大文件不稳 ( TLS 重置 / 无进度 )。
 * 用 Node fetch + 流式写盘, 带重试与进度。
 */
import fs from 'node:fs';
import path from 'node:path';

const ROOT = process.cwd();
const MODELS_DIR = path.join(ROOT, 'src-tauri', 'resources', 'models');
const FFMPEG_DIR = path.join(ROOT, 'src-tauri', 'resources', 'ffmpeg');

const MODELS = [
  ['https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/tokenizer.json', 'tokenizer.json'],
  ['https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/text_model_quantized.onnx', 'text_model.onnx'],
  ['https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/vision_model_quantized.onnx', 'vision_model.onnx'],
  ['https://huggingface.co/deepghs/insightface/resolve/main/buffalo_s/det_500m.onnx?download=true', 'det_500m.onnx'],
  ['https://huggingface.co/deepghs/insightface/resolve/main/buffalo_s/w600k_mbf.onnx?download=true', 'w600k_mbf.onnx'],
];

// ffmpeg sidecar: 从 Lap 官方 release 取 Windows x64 版
const FFMPEG_BASE = 'https://github.com/julyx10/lap/releases/download/v0.3.3';
const FFMPEG = [
  [FFMPEG_BASE + '/ffmpeg-x86_64-pc-windows-msvc.exe', 'ffmpeg-x86_64-pc-windows-msvc.exe'],
  [FFMPEG_BASE + '/ffprobe-x86_64-pc-windows-msvc.exe', 'ffprobe-x86_64-pc-windows-msvc.exe'],
];

async function fetchWithRetry(url, tries = 3) {
  let last;
  for (let i = 0; i < tries; i++) {
    try {
      const r = await fetch(url, { headers: { 'user-agent': 'curl/8' }, redirect: 'follow' });
      if (!r.ok) throw new Error('HTTP ' + r.status);
      return r;
    } catch (e) {
      last = e;
      console.log(`    retry ${i + 1}/${tries}: ${e.message}`);
      await new Promise((r) => setTimeout(r, 3000));
    }
  }
  throw last;
}

async function download(url, dest, label) {
  if (fs.existsSync(dest) && fs.statSync(dest).size > 0) {
    console.log(`SKIP  ${label} (已存在 ${(fs.statSync(dest).size / 1048576).toFixed(1)} MB)`);
    return;
  }
  console.log(`GET   ${label}`);
  const res = await fetchWithRetry(url);
  const total = Number(res.headers.get('content-length') || 0);
  const ws = fs.createWriteStream(dest);
  let got = 0, lastPct = -1;
  const t0 = Date.now();
  for await (const chunk of res.body) {
    ws.write(chunk);
    got += chunk.length;
    const pct = total ? Math.floor((got / total) * 100) : 0;
    if (pct >= lastPct + 20) {
      lastPct = pct;
      const mbps = (got / 1048576 / ((Date.now() - t0) / 1000)).toFixed(1);
      console.log(`  ${pct}%  ${(got / 1048576).toFixed(1)}/${(total / 1048576).toFixed(1)} MB  ${mbps} MB/s`);
    }
  }
  ws.end();
  await new Promise((r) => ws.on('finish', r));
  console.log(`DONE  ${label}  ${(got / 1048576).toFixed(1)} MB in ${((Date.now() - t0) / 1000).toFixed(0)}s`);
}

fs.mkdirSync(MODELS_DIR, { recursive: true });
fs.mkdirSync(FFMPEG_DIR, { recursive: true });

console.log('=== AI 模型 ===');
for (const [url, name] of MODELS) {
  await download(url, path.join(MODELS_DIR, name), name);
}

console.log('\n=== ffmpeg sidecar ===');
for (const [url, name] of FFMPEG) {
  try {
    await download(url, path.join(FFMPEG_DIR, name), name);
  } catch (e) {
    console.log(`WARN  ${name} 下载失败: ${e.message}`);
    console.log('      (ffmpeg 缺失只影响视频缩略图, 不阻塞本次编译验证)');
  }
}

console.log('\n=== 校验 ===');
for (const [, name] of MODELS) {
  const p = path.join(MODELS_DIR, name);
  const ok = fs.existsSync(p) && fs.statSync(p).size > 1000;
  console.log(`${ok ? 'OK ' : 'MISS'} ${name}${ok ? '  ' + (fs.statSync(p).size / 1048576).toFixed(1) + ' MB' : ''}`);
}
for (const [, name] of FFMPEG) {
  const p = path.join(FFMPEG_DIR, name);
  const ok = fs.existsSync(p) && fs.statSync(p).size > 1000;
  console.log(`${ok ? 'OK ' : 'MISS'} ${name}`);
}
