# 实现日志（implementation-log）

按阶段记录《炼金药铺》从键盘原型到可上线展示版本的完善过程。

## 阶段 1：策划与架构（docs/design.md）

- 通读全部源码（`src/game/*.rs`、`src/bin/capture.rs`、README、交接文档）。
- 产出策划案 `docs/design.md`：问题定位、统一输入架构、UI Kit、新手引导、音效方案、平衡说明。

## 阶段 2：统一输入通道（actions.rs）

- 新增 `UiAction` 消息 + `InputSet`：键盘与鼠标都翻译成消息，各业务系统只消费一次（`.after(InputSet)` 保证同帧可读）。
- 新增 `TempControl` 按住状态：键盘 `↑/↓`、`W/S` 与鼠标按住「升温/降温」按钮完全等价。
- 原键盘快捷键（Enter 接单 / Space 搅拌 / Tab 循环面板 / 1..0 购料 / 1..4 升级 / `[` `]` 调数量）全部保留为消息路径。

## 阶段 3：UI Kit 与真实面板（ui_kit.rs / ui.rs）

- 调色板 `C_*`、`ButtonKind`、`spawn_button`、`refresh_buttons`（按可购性自动置灰）、卡片、图标徽章、进度条。
- 重写 `ui.rs`：
  - 标题屏（开门营业按钮）
  - HUD：金 / 声 / 日 / 收 / 时 图标徽章 + 市场 / 升级 / 配方书 / 暂停按钮
  - 柜台 + 坩埚炼药面板：温度刻度、理想区间色带、烧焦预警、搅拌提示、接单 / 降温 / 搅拌 / 升温按钮
  - 市场面板：12 行材料（色块 + 名称 + 阶 + 单价 + 库存 + 购入按钮 + 数量 ±）
  - 升级面板、配方书、日结算（品质条形图 + 明日目标 + 继续 / 回标题）、破产 / 胜利、暂停覆盖层
- 非活动面板 `Visibility::Hidden`。

## 阶段 4：玩法系统重写

- `customers.rs`：接单走 UiAction、顾客气泡（Waiting/Served/Leaving + 低耐心红字）、等待 / 服务 / 离开动画、出货后满意离场。
- `brewing.rs`：`TempControl` 控温（键盘 + 鼠标按住）、`UiAction::Stir`、品质统计、音效请求。
- `economy.rs`（日钟暂停门控、结算走 UiAction）、`panels.rs`（购买走 UiAction + 音效）、`core.rs`（注册新插件、标题走 UiAction）、`resources.rs`（Paused/TempControl/TutorialSettings/day_quality/purchases）。
- `visual.rs`：坩埚液面随温度变色。

## 阶段 5：修复核心潜伏 bug

- **顾客永远到不了柜台**：原 `move_customers` 的浮动动画把 y 重置为移动前的 `cur.y`，顾客被钉在原地，订单 / 炼药循环从未真正跑通。改为在路径位置之上叠加 bob，顾客现在能走完 Walking → Waiting → Served → Leaving 全流程。
- **顾客交互区被右侧面板遮挡**（从原版继承）：柜台 / 顾客排队位置（世界 x=300 → 屏幕 x=940）整片落在右侧炼药面板（屏幕 x≥850）之下，顾客在柜台全程不可见。把家具整体左移 515（墙 / 地板保留铺满），柜台移到屏幕 x≈425，排队顾客与气泡现在完整可见。
- **面板打不开**：`OpenPanel` 消息没有任何系统消费它来切换 GameScreen 状态（只有按钮 / 按键写入，状态从不改变）。新增 `panel_navigation` 系统，成为唯一的状态切换点。
- 编译修复若干：ParamSet ≤8 参数、`Without<BrewHint>` 互斥、`BorderRadius` 是 Node 字段、`BorderColor::all()`、`Text(pub String)`、`AppExit` 是 Message。

## 阶段 6：音频（audio.rs，程序合成）

- `Cargo.toml` 启用 `bevy/wav`（rodio/hound）。
- 程序合成 click / stir / success / error / burn / coin 六种 SFX + 8 秒五声音阶 BGM 循环（音量 0.16，`PlaybackMode::Loop`；SFX 用 `Despawn`）。
- 全部通过 `AudioSource { bytes: Arc<[u8]> }` 内存注册，无外部资源；离屏 capture 下不崩溃。

## 阶段 7：新手引导（tutorial.rs）

- 8 步提示（接单 → 控温 → 搅拌 → 出货 → 市场 → 购料 → 升级 → 祝语），只提示不拦截。
- 目标按钮脉冲高亮；`TutorialSettings.enabled` 供 capture 关闭保证确定性。

## 阶段 8：验证

- `cargo check --all-targets` 全绿、0 警告。
- 离屏 capture（480 帧 ≈ 60s 真实时间）逐项验证：
  - 标题 → 开门营业 ✓
  - 顾客走进 → 排队 → 气泡 → 接单 → 服务 → 炼药 → 满意离场 ✓（帧分析确认顾客体色 / 气泡可见）
  - 市场（12 行 + 页脚完整不溢出，按钮按可购性置灰）✓
  - 升级 / 配方书 / 日结算（品质条形图 + 继续 / 回标题）✓
  - 暂停：帧差分确认覆盖层出现后画面完全冻结、恢复后继续 ✓
  - 强制日结算 → 第二天开始 ✓
- 窗口版 `cargo run` 启动验证（有窗口、正常渲染、BGM 可闻）。之前 `cargo run` 报“无法确定运行哪个二进制”导致窗口版起不来的问题：临时调试二进制 `src/bin/debug.rs` 引发多二进制歧义，已删除并设 `default-run = "godogen-apothecary"`。

## 已知噪音

- **ICU4X 刷屏**：`No segmentation model for complex script: Chinese/Japanese` 每秒大量输出。定位为 parley 0.9.0 使用 `WordSegmenter::new_for_non_complex_scripts`，从不调用 `with_japanese_dictionary()`，`ja` 载荷未被加载——依赖栈固有行为，无法通过 `icu_segmenter` 特性消除，也不可被 `RUST_LOG` 过滤（直接输出）。**文字渲染完全正常**（标题 / HUD / 面板均已逐帧确认），仅日志噪音，不影响游戏运行与发布。

## 阶段 9：交付

- `README.md`：真实操作说明（鼠标 + 键盘双通道、面板软暂停、暂停菜单、音频来源）。
- `docs/proof.mp4`：autopilot 演示视频（16s / 720p）。
- 提交并推送 GitHub。
