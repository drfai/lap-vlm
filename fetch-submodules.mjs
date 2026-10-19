/**
 * 通过 api.github.com 拉取 git submodule 的 tarball 并解压到目标路径。
 *
 * 为什么这么做: `git clone https://github.com/...` 的 git 协议在本机连不上
 * (Connection reset / Empty reply), 但 api.github.com 的 HTTPS 一直可达。
 * 主仓库当初也是走了某种可达路径才 clone 成功的。
 *
 * 注意: submodule 必须 checkout 到 .gitmodules 里指定的那个 commit,
 * 不能直接用分支 HEAD —— build.rs 依赖特定版本的 API/结构。
 */
import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';

const REPO = process.cwd();

// submodule 路径 -> [owner/repo, 目标 commit]
const SUBMODULES = {
  'src-tauri/third_party/LibRaw':        ['LibRaw/LibRaw',                        'b860248a89d9082b8e0a1e202e516f46af9adb29'],
  'src-tauri/third_party/libde265':      ['strukturag/libde265',                  '3cd9fbf15ae30edafa22a29fdd8a89355ad52ad0'],
  'src-tauri/third_party/libheif':       ['strukturag/libheif',                   '62f1b8c76ed4d8305071fdacbe74ef9717bacac5'],
  'src-tauri/third_party/libjpeg-turbo': ['libjpeg-turbo/libjpeg-turbo',          '9217719d3a58633923b096af4c1d50d304768a64'],
};

function sh(cmd) {
  return execSync(cmd, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], maxBuffer: 64 * 1024 * 1024 });
}

for (const [subPath, [repo, commit]] of Object.entries(SUBMODULES)) {
  const dest = path.join(REPO, subPath);
  const already = fs.existsSync(path.join(dest, 'CMakeLists.txt')) ||
                  fs.readdirSync(dest).length > 0;

  if (already) {
    console.log(`SKIP  ${subPath} (非空)`);
    continue;
  }

  console.log(`FETCH ${repo} @ ${commit.slice(0, 8)} -> ${subPath}`);

  // tarball 里有个顶层目录 <repo>-<commit>/, 解压后要剥掉
  const tmp = path.join(REPO, '.submodule-tmp');
  fs.rmSync(tmp, { recursive: true, force: true });
  fs.mkdirSync(tmp, { recursive: true });

  const url = `https://api.github.com/repos/${repo}/tarball/${commit}`;
  const tarPath = path.join(tmp, 'sub.tar.gz');

  // 用 curl: Node fetch 在大文件上慢且易超时
  sh(`curl.exe -L --fail --connect-timeout 20 --max-time 600 -o "${tarPath}" "${url}"`);
  console.log(`  downloaded ${(fs.statSync(tarPath).size / 1048576).toFixed(1)} MB`);

  sh(`tar.exe -xzf "${tarPath}" -C "${tmp}"`);
  const extracted = fs.readdirSync(tmp).find((n) => n !== 'sub.tar.gz');
  if (!extracted) throw new Error(`解压后未找到目录: ${tmp}`);

  // 移动到目标位置
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  fs.cpSync(path.join(tmp, extracted), dest, { recursive: true });
  fs.rmSync(tmp, { recursive: true, force: true });

  const files = fs.readdirSync(dest);
  console.log(`  OK ${files.length} entries, e.g. ${files.slice(0, 5).join(', ')}`);
}

console.log('\n全部 submodule 就绪。');
