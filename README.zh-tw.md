# 羅技 G 系列滑鼠 — Omarchy 外掛 (oma-logitech-g-mouse)

[English](README.md) | **繁體中文**

專為 **Logitech G 系列電競滑鼠**（包含 G PRO X2 SUPERSTRIKE、G PRO X SUPERLIGHT 1/2、G502 系列等）設計的 [Omarchy Quattro](https://omarchyplugins.com/develop.html) 狀態列外掛（Bar Widget + Control Panel）。

透過 Linux 原生 `/dev/hidraw` 節點與 Logitech **HID++ 2.0** 協議直接通訊，無需官方 G HUB，即可在 Omarchy 桌面環境中享有完整的遙測讀取、靈敏度、回報率與類比開關調校功能。

## 需求

- Omarchy Quattro。
- 已安裝 Bun，且 Omarchy Shell 的 `PATH` 可找到 `bun`。

若 Bun 不在 Shell 的 `PATH` 中，請在外掛設定的 **Path to Bun** 填入 Bun 執行檔的絕對路徑。

---

## 功能

### 狀態列小工具

- **電量與狀態**：顯示 `99% 󰁹`、充電圖示（`󰂅`）或離線狀態（`Off 󰍽`）。
- **右鍵切換百分比**：切換圖示模式（`󰁹`）與百分比模式（`99% 󰁹`）；設定儲存在 `~/.config/omarchy/shell.json`。
- **主題整合**：使用系統 `foreground` 色彩；面板開啟時不改變狀態列顏色。

### 控制面板

左鍵開啟含兩個分頁的 `KeyboardPanel`：

- **`Sensor (DPI / Polling)`**
  - **DPI 預設**：依滑鼠板載預設清單產生滿寬按鈕。
  - **DPI 滑桿**：從 `100` 到偵測到的感應器最大值，每次調整 50 DPI。
  - **回報率**：若 HID++ Feature `0x8061` 支援，可選擇 `125` / `250` / `500` / `1K` / `2K` / `4K` / `8K Hz`。
- **`Buttons (Analog HITS)`** _(僅在具備類比開關的滑鼠顯示)_
  - 左右鍵分別設定 **Actuation Point**（`1`–`10`）、**Rapid Trigger**（`1`–`5`）與 **Click Haptics**（`1`–`6`）。
  - 每個滑桿下方有數字刻度。
  - 樂觀狀態更新可避免硬體寫入期間滑桿回彈。

---

## 安裝與解除安裝（Install & Uninstall）

### 安裝（Install）

#### 方法 A：透過 Omarchy CLI 遠端安裝（Git 安裝）

若本專案已推送至公開或私有 Git 倉庫，可直接一行指令安裝並啟用：

```bash
omarchy plugin add https://github.com/ttymayor/oma-logitech-g-mouse.git --enable
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

## 開發與修改套用

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

## 獨立 CLI 命令列控制

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

## HID++ 2.0 協議對照

| 功能                     | Feature ID | Index  | 備註                                                                |
| :----------------------- | :--------- | :----- | :------------------------------------------------------------------ |
| **Device Name**          | `0x0005`   | `0x03` | 行銷型號名稱動態讀取（如 `PRO X2 SUPERSTRIKE`、`PRO X SUPERLIGHT`） |
| **Unified Battery**      | `0x1004`   | `0x06` | 電量百分比、電量等級、即時充電狀態                                  |
| **Extended DPI**         | `0x2202`   | `0x09` | 感應器參數讀寫（支援至 32K DPI）與 LOD 狀態                         |
| **Onboard Profiles**     | `0x8100`   | `0x0e` | 板載設定檔切換、Host Mode 模式解鎖                                  |
| **HITS Analog Switches** | `0x1B0C`   | `0x0c` | 類比觸發（1-10）、Rapid Trigger（1-5）、震動力度（1-6）             |
| **Extended Report Rate** | `0x8061`   | `0x0d` | 高頻輪詢率控制（125Hz 至 8000Hz）                                   |

---

## 權限與 Udev 規則

> **提示：** 在 Arch Linux、Omarchy 以及絕大多數現代基於 systemd 的桌面環境中，**你通常「不需要」執行這個步驟！**  
> 系統內建的 `systemd-logind` 會自動透過 `uaccess` 動態存取控制清單（ACL），直接將 `/dev/hidraw*` 的讀寫權限（`user:username:rw-`）授予當前登入桌面的使用者帳號。

### 什麼情況下才需要執行此步驟？

只有在遇到以下少數特殊情況時才需要設定：

1. 執行 `bun run logitech-g-daemon.ts --once` 時噴出 **`EACCES: Permission denied`**（權限不足）錯誤。
2. 使用沒有配置 `systemd-logind` 的極簡發行版、Docker 容器環境，或在沒有圖形登入會話（無 seat0）的純終端機/伺服器環境中執行。

若遇到上述權限不足的情況，可建立以下 udev 規則賦予所有人讀寫權限：

```bash
sudo tee /etc/udev/rules.d/42-logitech-mouse.rules << 'EOF'
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="046d", MODE="0666"
EOF

sudo udevadm control --reload && sudo udevadm trigger
```

---

## 授權條款

MIT License.  
Logitech、G PRO、SUPERSTRIKE 為羅技公司之註冊商標，本專案為第三方獨立開源外掛，與羅技官方無隸屬關係。
