# MHGU 逐次磨刀与翻滚取消

v0.1.1 · MHGU 1.4.0 · Recomp / Forge 插件

[下载安装包](https://github.com/Akumi-xu/MHGU-Progressive-Sharpen/releases/download/v0.1.1/MHGU-Progressive-Sharpen-v0.1.1-Recomp.zip) · [Release v0.1.1](https://github.com/Akumi-xu/MHGU-Progressive-Sharpen/releases/tag/v0.1.1)

## 功能

- 每完成一下磨刀摩擦，立即恢复一部分斩味。
- 普通四次为 **37 + 38 + 37 + 38 = 150 点**，达到武器上限后不再增加。
- 快磨一次恢复 150 点，慢磨五次各 30 点；特殊磨刀道具按原版 100 点分摊。
- 起手、摩擦、收尾均可按回避键取消，保留已恢复的斩味；首次恢复时按原版规则消耗一次道具。
- 回避方向和体力由原版处理；刚刃等完成效果仍需磨刀完成。
- v0.1.1 修复满斩后反复出现满值提示的问题；已经满斩时不重复触发。

只缺 35 点斩味时，第一下即可补满；本 MOD 分摊原版恢复点数，不把每次的实际缺口强制分成四份。

## 安装与卸载

关闭游戏，在 `mhgurecomp.exe` 旁的 `mods` 中新建 `MHGU_PROGRESSIVE_SHARPEN` 文件夹，将安装包内的 `romfs` 放进去：

```text
mods/MHGU_PROGRESSIVE_SHARPEN/romfs/nativeNX/plugins/progressive_sharpen.nro
```

Recomp 自带 Forge，不需要额外 Switch 加载器。同名 NRO 只装一份。卸载时关闭游戏，删除本 MOD 文件夹即可。本版无 ini 或调参菜单。

### 与 World Sharpness Gauge r65 共存

在游戏 MOD 菜单或 `mods/load_order.txt` 中，让 `world-sharpness-gauge` **先于** `MHGU_PROGRESSIVE_SHARPEN` 加载。r65 会校验原版磨刀恢复函数，反序会导致斩味条因入口已被挂钩而自行停用。仓库不包含第三方斩味条 MOD。

## 验证范围

Windows MHGURecomp 0.16.0 / Forge 1.5.1 / GU 1.4.0，太刀：分段恢复、快磨/慢磨、各阶段取消、只扣一次道具、满斩上限、方向、体力以及本机现有 MOD 共存已测试。

满值通知实测：差 10 点时只通知一次；已满时不通知。恢复总量保持原版规则。

未逐项实测全部武器与六风格组合；安卓、Eden、Switch、联机未验证。偏移绑定 GU 1.4.0，遇到函数特征不符会停止安装 hook。

## 构建与自检

克隆到纯 ASCII 路径。准备 Rust nightly + rust-src、cargo-mhgu-forge 1.5.0、ARM GNU Toolchain 和 Python 3，执行：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ./06_tools/build.ps1 -Project progressive_sharpen
rustc --test ./01_plugins/progressive_sharpen/src/session.rs -o ./session_tests.exe
./session_tests.exe
```

构建脚本自动检查 NRO 重定位；本版构建与 3 项自检通过。源码为本版快照，不包含游戏文件、生成的游戏 C 源码、存档或密钥。

依赖：[Forge](https://github.com/Fexty12573/forge) / `mhgu-forge`。开发使用 ChatGPT Codex 辅助。

安装包 SHA-256：`19486a4bb011db869cd9224a6915b4316aff0eed4106e5e8429e5d24b4b38456`

仓库保存展开的源码与构建材料；安装包统一从 Release 下载。历史版本及标签保持原样。
