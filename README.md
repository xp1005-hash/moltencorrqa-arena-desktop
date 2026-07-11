# MoltenCorrQA 专家盲评 — Windows 桌面客户端（Tauri v2）

薄壳客户端：主窗口直接加载远程 Arena `http://47.101.156.148:8200/`（4070 经 frp 发布），
本地不打包任何题目/答案数据。

## 架构与更新策略

| 层 | 更新方式 |
|---|---|
| 评审前端 + 题库 | 全在 4070 服务端，改 `index.html`/`arena_data.json` 即全员生效，无需发版 |
| 客户端壳（本工程） | GitHub Release 自动更新：启动时 Rust 侧检查 `latest.json` → 弹窗确认 → 下载安装 → 重启 |

- 远程页面没有 IPC 权限，更新逻辑全部在 Rust（`src/main.rs` 的 `check_update`），
  用 `tauri-plugin-updater` + `tauri-plugin-dialog`。
- 更新源：`https://github.com/xp1005-hash/nuclear-corrosion-ai/releases/latest/download/latest.json`
- `dist/index.html` 是离线兜底说明页（frontendDist）：远程加载失败时的提示 + 重试按钮。

## 发版流程

1. 改 `src-tauri/tauri.conf.json` 与 `src-tauri/Cargo.toml`、`package.json` 的 `version`（三处一致）
2. 提交后打 tag 并推送：

   ```bash
   git tag arena-v0.2.0 && git push origin arena-v0.2.0
   ```

3. GitHub Actions（`.github/workflows/arena-desktop.yml`，windows-latest）自动构建
   NSIS 安装包 + 签名 + `latest.json` 并发布 Release；已装用户下次启动即收到更新弹窗。

## 签名密钥

- 私钥：Mac 本机 `~/.tauri/arena_updater.key`（无密码），**务必备份**——丢失后老客户端无法再自动更新
- 公钥：已嵌入 `src-tauri/tauri.conf.json` 的 `plugins.updater.pubkey`
- CI secrets：`TAURI_SIGNING_PRIVATE_KEY`（私钥文件内容）、`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`（空串）

## 本地开发

```bash
npm install
npm run tauri dev     # 需要 Rust 工具链；Windows 上需要 WebView2（Win10+ 自带）
npm run tauri build   # 本地出安装包（macOS 上出 dmg，Windows 上出 nsis exe）
```
