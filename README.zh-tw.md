# 羅技 G 系列滑鼠 — Omarchy 外掛 (oma-logitech-g-mouse)

[English](README.md) | **繁體中文**

專為 **Logitech G 系列電競滑鼠**（包含 G PRO X2 SUPERSTRIKE、G PRO X SUPERLIGHT 1/2、G502 系列等）設計的 [Omarchy Quattro](https://omarchyplugins.com/develop.html) 狀態列外掛（Bar Widget + Control Panel）。

透過 Linux 原生 `/dev/hidraw` 節點與 Logitech **HID++ 2.0** 協議直接通訊，無需官方 G HUB，即可在 Omarchy 桌面環境中享有完整的遙測讀取、靈敏度、回報率與類比開關調校功能。

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
- 🏷️ **Tab 2: `Buttons (Analog HITS)`** *(若滑鼠具備類比開關則自動顯示)*
  - **左鍵與右鍵獨立雙區**，各具備 3 組原生單色滑桿：
    1. **Actuation Point（下壓觸發深度）**：`1` ~ `10` 級（0.1mm ~ 1.0mm 觸發行程）
    2. **Rapid Trigger（快速復位重置行程）**：`1` ~ `5` 級（0.1mm ~ 0.5mm 抬起即重置）
    3. **Click Haptics（震動點擊回饋力度）**：`1` ~ `6` 級觸覺反饋
  - **刻度數字標尺**：滑桿正下方對齊呈現具體數字（`1..10`、`1..5`、`1..6`），並隨目前選中檔位加粗高亮。
  - **零回彈防延遲技術**：採用樂觀狀態更新（Optimistic Updates），滑動與放開時指針平滑停駐，絕無跳動或彈回原位的延遲感。

---

## 📦 安裝與解除安裝（Install & Uninstall）

### 安裝（Install）

#### 方法 A：透過 Omarchy CLI 遠端安裝（Git 安裝）
若本專案已推送至公開或私有 Git 倉庫，可直接一行指令安裝並啟用：

```bash
omarchy plugin add https://github.com/<username>/oma-logitech-g-mouse.git --enable
omarchy restart shell
```

#### 方法 B：本地手動安裝（從本專案原始碼部署）
如果是本地端開發或從原始碼資料夾安裝：

```bash
# 1. 複製外掛檔案至 Omarchy 使用者外掛目錄
mkdir -p ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse
cp -a * ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse/

# 2. 驗證外掛結構規範
omarchy plugin validate ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse

# 3. 讓 Shell 重新掃描外掛清單並在狀態列右側啟用
omarchy-shell shell rescanPlugins
omarchy plugin enable tantuyu.logitech-g-mouse --section right

# 4. 重啟 Omarchy Shell 載入 QML 元件
omarchy restart shell
```

---

### 解除安裝（Uninstall）

#### 方法 A：透過 Omarchy CLI 移除
```bash
# 停用外掛並從系統中完全移除
omarchy plugin disable tantuyu.logitech-g-mouse
omarchy plugin remove tantuyu.logitech-g-mouse --yes
omarchy restart shell
```

#### 方法 B：手動完全清除
```bash
# 1. 停用外掛
omarchy plugin disable tantuyu.logitech-g-mouse

# 2. 刪除外掛目錄與暫存狀態快取檔
rm -rf ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse
rm -f /tmp/omarchy-logitech-g-mouse.json

# 3. 通知 Shell 重新掃描並重啟
omarchy-shell shell rescanPlugins
omarchy restart shell
```

---

## 🛠️ 開發與修改套用

- **修改了 QML / 介面（`Panel.qml`、`BarWidget.qml`、`Model.js`）**：
  因為 Omarchy 關閉了 QML 自動檔案監聽以維持系統穩定，修改後**必須重啟 Shell**：
  ```bash
  cp -a . ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse/ && omarchy restart shell
  ```
- **只修改了 TypeScript 驅動（`logitech-g-daemon.ts`）**：
  **不需要重啟 Shell**！下一次點擊按鈕或定時輪詢時會由 Bun 自動即時載入執行。
- **一鍵驗證、同步與重啟指令**：
  ```bash
  omarchy plugin validate . && cp -a . ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse/ && omarchy restart shell
  ```

---

## 💻 獨立 CLI 命令列控制

核心驅動腳本 `logitech-g-daemon.ts` 亦支援作為獨立命令列工具使用：

```bash
# 讀取完整狀態（JSON 輸出）
bun run logitech-g-daemon.ts --once

# 設定 DPI（支援 100 ~ 32,000）
bun run logitech-g-daemon.ts --set-dpi 1600

# 設定回報率（125 / 250 / 500 / 1000 / 2000 / 4000 / 8000 Hz）
bun run logitech-g-daemon.ts --set-rate 4000

# 設定下壓觸發點 Actuation Point（1~10 級）
bun run logitech-g-daemon.ts --set-actuation 4 --left
bun run logitech-g-daemon.ts --set-actuation 6 --right

# 設定快速復位行程 Rapid Trigger（1~5 級）
bun run logitech-g-daemon.ts --set-rt 2 --left

# 設定震動點擊回饋 Click Haptics（1~6 級）
bun run logitech-g-daemon.ts --set-haptics 5 --left
```

---

## 🔬 底層 HID++ 2.0 協議對照

| 功能 | Feature ID | Index | 備註 |
| :--- | :--- | :--- | :--- |
| **Device Name** | `0x0005` | `0x03` | 行銷型號名稱動態讀取（如 `PRO X2 SUPERSTRIKE`、`PRO X SUPERLIGHT`） |
| **Unified Battery** | `0x1004` | `0x06` | 電量百分比、電量等級、即時充電狀態 |
| **Extended DPI** | `0x2202` | `0x09` | 感應器參數讀寫（支援至 32K DPI）與 LOD 狀態 |
| **Onboard Profiles** | `0x8100` | `0x0e` | 板載設定檔切換、Host Mode 模式解鎖 |
| **HITS Analog Switches** | `0x1B0C` | `0x0c` | 類比觸發（1-10）、Rapid Trigger（1-5）、震動力度（1-6） |
| **Extended Report Rate** | `0x8061` | `0x0d` | 高頻輪詢率控制（125Hz 至 8000Hz） |

---

## 🔒 權限與 Udev 規則

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
