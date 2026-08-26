# Logitech G PRO X2 SUPERSTRIKE — Omarchy 外掛

[English](README.md) | **繁體中文**

專為 **Logitech G PRO X2 SUPERSTRIKE** 電競類比滑鼠設計的 [Omarchy Quattro](https://omarchyplugins.com/develop.html) 狀態列外掛（Bar Widget + Control Panel）。

透過 Linux 原生 `/dev/hidraw` 節點與 Logitech **HID++ 2.0** 協議直接通訊，無需官方 G HUB，即可在 Omarchy 桌面環境中享有完整的遙測讀取與硬體調校功能。

---

## ✨ 主要功能特點

### 1. 狀態列小工具（Status Bar Widget）
- **即時電量與狀態**：顯示如 `99% 󰁹`，支援充電中動態圖示（`󰂅`）及離線提示（`Off 󰍽`）。
- **右鍵切換百分比**：在狀態列圖示上**點擊右鍵**即可一鍵切換「精簡圖示模式（`󰁹`）」或「圖示+百分比模式（`99% 󰁹`）」，設定自動持久化儲存於 `~/.config/omarchy/shell.json`。
- **純粹單色系設計**：恆常鎖定系統 `foreground` 色彩，點開面板時狀態列按鈕不會閃爍變色。

### 2. 雙分頁控制面板（2-Tab Control Panel）
點擊狀態列按鈕即以懸浮視窗（KeyboardPanel）展開，分為兩大清楚主題，無垂直高度遮蔽問題：

- 🏷️ **Tab 1: `Sensor (DPI / Polling)`（感應器與回報率）**
  - **DPI 常用預設**：5 顆 100% 滿版等寬按鈕（`800` / `1200` / `1600` / `2400` / `3200`），點擊即設。
  - **DPI 連續微調滑桿**：支援從 `100` 至最高 `32,000 DPI` 連續無級微調（步進 50 DPI），附 `100` / `16,000` / `32,000` 刻度錨點。
  - **回報率設定（Polling Rate）**：7 顆 100% 滿版等寬按鈕（`125` / `250` / `500` / `1K` / `2K` / `4K` / `8K Hz`），官方原生協議即時切換。
- 🏷️ **Tab 2: `Buttons (Analog HITS)`（左右鍵類比開關全覽）**
  - **左鍵與右鍵獨立雙區**，各具備 3 組原生單色滑桿：
    1. **Actuation Point（下壓觸發深度）**：`1` ~ `10` 級（0.1mm ~ 1.0mm 觸發行程）
    2. **Rapid Trigger（快速復位重置行程）**：`1` ~ `5` 級（0.1mm ~ 0.5mm 抬起即重置）
    3. **Click Haptics（震動點擊回饋力度）**：`1` ~ `6` 級觸覺反饋
  - **刻度數字標尺**：滑桿正下方對齊呈現具體數字（`1..10`、`1..5`、`1..6`），並隨目前選中檔位加粗高亮。
  - **零回彈防延遲技術**：採用樂觀狀態更新（Optimistic Updates），滑動與放開時指針平滑停駐，絕無跳動或彈回原位的延遲感。

---

## 🚀 安裝與啟用

### 1. 部署至 Omarchy 外掛目錄
將本專案同步至 `~/.config/omarchy/plugins/`：

```bash
mkdir -p ~/.config/omarchy/plugins/tantuyu.g-pro-x2-superstrike
cp -a * ~/.config/omarchy/plugins/tantuyu.g-pro-x2-superstrike/
```

### 2. 驗證與啟用
```bash
# 驗證外掛結構規範
omarchy plugin validate ~/.config/omarchy/plugins/tantuyu.g-pro-x2-superstrike

# 重新掃描外掛清單
omarchy-shell shell rescanPlugins

# 啟用並加入狀態列右側
omarchy plugin enable tantuyu.g-pro-x2-superstrike --section right

# 重啟 Shell 載入 QML 元件
omarchy restart shell
```

---

## 🛠️ 開發與修改套用

如果你自行修改了外掛檔案，可依修改類型採取以下方式套用：

- **修改了 QML / 介面（`Panel.qml`、`BarWidget.qml`、`Model.js`）**：
  因為 Omarchy 關閉了 QML 自動檔案監聽以維持系統穩定，修改後**必須重啟 Shell**：
  ```bash
  cp -a . ~/.config/omarchy/plugins/tantuyu.g-pro-x2-superstrike/ && omarchy restart shell
  ```
- **只修改了 TypeScript 驅動（`superstrike-daemon.ts`）**：
  **不需要重啟 Shell**！下一次點擊按鈕或定時輪詢時會由 Bun 自動即時載入執行。
- **一鍵驗證、同步與重啟指令**：
  ```bash
  omarchy plugin validate . && cp -a . ~/.config/omarchy/plugins/tantuyu.g-pro-x2-superstrike/ && omarchy restart shell
  ```

---

## 💻 獨立 CLI 命令列控制

核心驅動腳本 `superstrike-daemon.ts` 亦支援作為獨立命令列工具使用：

```bash
# 讀取完整狀態（JSON 輸出）
bun run superstrike-daemon.ts --once

# 設定 DPI（支援 100 ~ 32,000）
bun run superstrike-daemon.ts --set-dpi 1600

# 設定回報率（125 / 250 / 500 / 1000 / 2000 / 4000 / 8000 Hz）
bun run superstrike-daemon.ts --set-rate 4000

# 設定下壓觸發點 Actuation Point（1~10 級）
bun run superstrike-daemon.ts --set-actuation 4 --left
bun run superstrike-daemon.ts --set-actuation 6 --right

# 設定快速復位行程 Rapid Trigger（1~5 級）
bun run superstrike-daemon.ts --set-rt 2 --left

# 設定震動點擊回饋 Click Haptics（1~6 級）
bun run superstrike-daemon.ts --set-haptics 5 --left
```

---

## 🔬 底層 HID++ 2.0 協議對照

本外掛逆向並實作了 Logitech G PRO X2 Superstrike 的硬體功能表：

| 功能 | Feature ID | Index | 備註 |
| :--- | :--- | :--- | :--- |
| **Unified Battery** | `0x1004` | `0x06` | 電量百分比、電量等級、即時充電狀態 |
| **Extended DPI** | `0x2202` | `0x09` | 感應器參數讀寫（支援至 32K DPI）與 LOD 狀態 |
| **Onboard Profiles** | `0x8100` | `0x0e` | 板載設定檔切換、Host Mode 模式解鎖 |
| **HITS Analog Switches** | `0x1B0C` | `0x0c` | 類比觸發（1-10）、Rapid Trigger（1-5）、震動力度（1-6） |
| **Extended Report Rate** | `0x8061` | `0x0d` | 高頻輪詢率控制（125Hz 至 8000Hz） |

---

## 🔒 權限與 Udev 規則

若要在非 root 使用者權限下存取 `/dev/hidraw*`，建議安裝 udev 規則：

```bash
sudo tee /etc/udev/rules.d/42-logitech-superstrike.rules << 'EOF'
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="046d", MODE="0666"
EOF

sudo udevadm control --reload && sudo udevadm trigger
```

---

## 📄 授權條款

MIT License.  
Logitech、G PRO、SUPERSTRIKE 為羅技公司之註冊商標，本專案為第三方獨立開源外掛，與羅技官方無隸屬關係。
