<div align="center">

<p align="center">
  <img src="assets/heptapod_circle.svg" width="160" height="160" alt="Rotating Heptapod B Logogram" />
</p>

# 🌌 Louise (Dr. Louise Banks)
### Discreet, Keyboard-First Instant Desktop Translator for Windows

<p align="center">
  <a href="https://github.com/MarianQQQ/LouiseTranslator">
    <img src="https://readme-typing-svg.demolab.com?font=JetBrains+Mono&weight=600&size=16&duration=3000&pause=1000&color=00C0F0&center=true&vCenter=true&width=620&lines=Discreet%2C+keyboard-first+instant+desktop+translator.;Highlight+text+%2B+Alt%2BC+%E2%86%92+Instant+translation.;Screen+OCR+%2B+Alt%2BS+%E2%86%92+Snip+any+image%2C+video%2C+PDF.;Dual-Engine%3A+DeepL+Neural+AI+%2B+Google+Fallback.;Crafted+with+pure+Rust+%26+Slint+UI+(~60+MB+RAM)." alt="Louise Typing Animation" />
  </a>
</p>

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Slint UI](https://img.shields.io/badge/Slint_UI-v1.18-00C0F0.svg?style=for-the-badge&logo=qt)](https://slint.dev/)
[![Windows](https://img.shields.io/badge/Platform-Windows_10_%7C_11-0078D6.svg?style=for-the-badge&logo=windows)](https://microsoft.com)
[![DeepL](https://img.shields.io/badge/DeepL-Neural_AI_Supported-00b4d8.svg?style=for-the-badge)](https://www.deepl.com/)
[![Windows OCR](https://img.shields.io/badge/Windows_OCR-Offline_%26_Native-7928ca.svg?style=for-the-badge&logo=windows)](https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr)
[![License](https://img.shields.io/badge/License-MIT-22c55e.svg?style=for-the-badge)](LICENSE)
[![Privacy](https://img.shields.io/badge/Privacy-100%25_Local_%26_Zero_Telemetry-a855f7.svg?style=for-the-badge)](#-the-philosophy-behind-louise)

<br>

<p align="center">
  <img src="assets/divider.svg" width="70%" />
</p>

<br>

> *«Language is the first weapon drawn in a conflict... or the first hand extended in understanding.»*  
> — **Dr. Louise Banks**, *Arrival* (2016)

<br>

[**The Philosophy**](#-the-philosophy-behind-louise) • [**Interface Showcase**](#-interface-showcase) • [**Key Features**](#-key-features) • [**Screen OCR & Language Packs**](#-screen-ocr--windows-language-packs) • [**Dual-Engine Architecture**](#-dual-engine-architecture) • [**Shortcuts**](#-shortcuts-cheatsheet) • [**System Architecture**](#-system-architecture) • [**Quick Start**](#-quick-start--build)

</div>

<br>

---

## 🎬 The Philosophy Behind Louise

<div align="center">
  <table border="0" style="border: none; background: transparent;">
    <tr>
      <td width="48%" align="center" style="vertical-align: middle; border: none; padding: 10px;">
        <img src="assets/louise_banks.jpg" alt="Dr. Louise Banks deciphering Heptapod B" style="border-radius: 10px; box-shadow: 0 12px 36px rgba(0,0,0,0.7);" width="100%" />
      </td>
      <td width="52%" style="vertical-align: middle; border: none; padding-left: 20px; text-align: left;">
        <h3>👩‍🔬 Dr. Louise Banks — The Heart of Communication</h3>
        <p>
          <i>«If you could see your whole life from start to finish, would you change things?»</i>
        </p>
        <p>
          In Denis Villeneuve's masterpiece <b>«Arrival»</b> (adapted from Ted Chiang's <i>«Story of Your Life»</i>), linguistics professor <b>Dr. Louise Banks</b> (played by Amy Adams) is summoned when enigmatic alien vessels hover silently over the planet. Where military factions prepare for confrontation, Louise enters the unknown with stillness, patience, and profound empathy.
        </p>
        <p>
          By deciphering their circular ink logograms (<i>Heptapod B</i>), she discovers that learning a new language isn't just about translating words — it fundamentally alters how we experience time, thought, and our connection to one another.
        </p>
        <p>
          <b>Louise Translator</b> is built upon this exact ethos: an elegant, quiet, distraction-free desktop companion that dissolves communication barriers instantly without ever getting in your way.
        </p>
      </td>
    </tr>
  </table>
</div>

<p align="center">
  <img src="assets/divider.svg" width="60%" />
</p>

---

## 📸 Interface Showcase

<p align="center">
  <img src="assets/ui_main.png" alt="Louise Main Translation Window" width="47%" style="border-radius: 8px; box-shadow: 0 8px 30px rgba(0,0,0,0.5);" />
  &nbsp;&nbsp;
  <img src="assets/ui_settings.png" alt="Louise Priority Languages Settings" width="47%" style="border-radius: 8px; box-shadow: 0 8px 30px rgba(0,0,0,0.5);" />
</p>
<p align="center">
  <i>Left: Minimalist Dark UI with Auto-detect, OCR viewfinder button, DeepL ⚡ badge, and inline actions. Right: Fast Priority Languages customization with SVG flags.</i>
</p>

---

## ✨ Key Features

- **⚡ Instant Selection Capture (`Alt + C`)**:
  Highlight foreign text in any Windows app (browser, Discord, IDE, PDF reader, Slack) and hit `Alt + C`. Louise appears next to your mouse with zero delay. If no text is selected, Louise discreetly remains silent without popping up empty windows.
- **📷 Offline Screen OCR Capture (`Alt + S` or In-App `OCR · Alt+S` Button)**:
  Translate text locked in images, YouTube videos, games, scanned PDFs, slides, or protected web pages. Hit `Alt + S` to freeze your multi-monitor screen, drag a selection box, and Louise instantly extracts and translates the text using offline, hardware-accelerated **Windows Media OCR**.
  - **Zero-Ghosting Instant Hide**: Louise teleports off-screen in 0ms before taking the screen snapshot, ensuring its own window never obscures your snip.
  - **Background Engine Warmup**: OCR libraries and language models pre-initialize quietly upon startup, making snips instantaneous.
  - **Smart Post-Processing**: Automatically normalizes mixed casing and corrects Cyrillic/Ukrainian character substitutions (`є`, `і`, `ї`, `ґ`).
- **🪟 Instant Window Toggle (`Alt + X`)**:
  Hit `Alt + X` to summon Louise near your cursor for manual input, or hit `Alt + X` again while open to instantly hide it back to tray.
- **🔄 In-Place Paste Replacement**:
  Replace foreign text with the translated text in one keystroke (`Paste as replacement`). Louise automatically focuses your previous window and simulates seamless replacement.
- **🧠 Dual-Engine Architecture (DeepL Neural AI + Google Translate)**:
  Choose between **DeepL Pro/Free API** (ultra-natural European phrasing, idiomatic accuracy) and **Google Translate** (instant, free, unlimited).
- **📊 Real-Time Dynamic Quota Tracking**:
  Directly polls the DeepL usage API to accurately calculate remaining characters for both Free tier (500k) and paid Pro / custom subscription limits.
- **🛡️ Fail-Safe Auto Fallback**:
  If DeepL encounters network drops, quota exhaustion, or unsupported dialects, Louise automatically falls back to Google Translate and notifies you via a sleek toast banner.
- **🌐 Default Auto-Detection**:
  The source language always defaults to smart **Auto-Detect** (`auto`), adapting instantly without resetting your selected target language.
- **🔒 100% Offline Privacy & Hardware-Encrypted DPAPI Keys**:
  Zero telemetry, zero external trackers. The executable ships with **zero hardcoded keys**. All user settings are stored locally in portable `config.json`, with API keys hardware-encrypted via Windows Data Protection API (DPAPI).
- **🖤 Bespoke Monochrome Dark Palette**:
  Deep `#101010` background, subtle borders, high-contrast typography, JetBrains Mono font, and buttery 180+ Hz zero-dead-zone window dragging.
- **⌨️ Fully Customizable Hotkeys with Layout Agility**:
  Separate customization for selection translation, screen OCR, and window toggle with release-to-commit key recording. Seamlessly supports both English and Ukrainian/Cyrillic keyboard layouts across all shortcuts including in-app <kbd>Ctrl</kbd>+<kbd>C</kbd>, <kbd>Ctrl</kbd>+<kbd>V</kbd>, <kbd>Ctrl</kbd>+<kbd>A</kbd>, and <kbd>Ctrl</kbd>+<kbd>X</kbd>.
- **🚀 Native Rust & Slint Performance**:
  No Chromium, no Electron, no JavaScript runtime. Tiny memory footprint (~60 MB RAM) and sub-15ms cold start.

<p align="center">
  <img src="assets/divider.svg" width="60%" />
</p>

---

## 📷 Screen OCR & Windows Language Packs

Louise utilizes native **Windows.Media.Ocr** APIs directly from the Windows OS. This delivers **100% offline, zero-latency, private recognition** without consuming any cloud tokens.

### 📥 Installing OCR Language Packs for Windows

Windows ships with your system display languages by default. If you frequently translate text from images in specific languages (such as Ukrainian, Polish, German, Japanese, etc.), you can install the official Microsoft OCR models in seconds.

#### Quick Method: PowerShell (Run as Administrator)

Open **PowerShell as Administrator** and run the one-line command for your desired language:

```powershell
# 🇺🇦 Ukrainian OCR
Add-WindowsCapability -Online -Name "Language.OCR~~~uk-UA~0.0.1.0"

# 🇵🇱 Polish OCR
Add-WindowsCapability -Online -Name "Language.OCR~~~pl-PL~0.0.1.0"

# 🇩🇪 German OCR
Add-WindowsCapability -Online -Name "Language.OCR~~~de-DE~0.0.1.0"

# 🇫🇷 French OCR
Add-WindowsCapability -Online -Name "Language.OCR~~~fr-FR~0.0.1.0"

# 🇪🇸 Spanish OCR
Add-WindowsCapability -Online -Name "Language.OCR~~~es-ES~0.0.1.0"

# 🇮🇹 Italian OCR
Add-WindowsCapability -Online -Name "Language.OCR~~~it-IT~0.0.1.0"

# 🇯🇵 Japanese OCR
Add-WindowsCapability -Online -Name "Language.OCR~~~ja-JP~0.0.1.0"

# 🇨🇳 Chinese (Simplified) OCR
Add-WindowsCapability -Online -Name "Language.OCR~~~zh-CN~0.0.1.0"

# 🇰🇷 Korean OCR
Add-WindowsCapability -Online -Name "Language.OCR~~~ko-KR~0.0.1.0"

# 🇺🇸 English (US) OCR
Add-WindowsCapability -Online -Name "Language.OCR~~~en-US~0.0.1.0"
```

#### How to Check Installed OCR Languages

Run this command in PowerShell to inspect which OCR packs are currently available on your PC:

```powershell
Get-WindowsCapability -Online | Where-Object { $_.Name -like "Language.OCR*" -and $_.State -eq "Installed" } | Select-Object Name
```

#### Alternative: Windows Settings GUI

1. Open **Windows Settings** (`Win + I`) → **Time & Language** → **Language & region**.
2. Find the desired language in the list (or click **Add a language**).
3. Click the **...** menu next to the language → **Language options**.
4. Under **Language features**, locate **Optical Character Recognition (OCR)** and click **Download / Install**.

---

## 🧠 Dual-Engine Architecture

<div align="center">

| Capability | ⚡ DeepL API (Neural AI) | 🌐 Google Translate (Built-in) |
| :--- | :--- | :--- |
| **Translation Engine** | Advanced Neural Machine Translation | Google Cloud Neural Translate |
| **Phrasing Quality** | Human-like nuance, idioms & natural flow | Fast, literal & reliable |
| **Setup Required** | Free or Pro API key (Auto-detects plan limit) | **None** (zero setup) |
| **Languages** | 30+ major languages (European, Asian) | 100+ global languages |
| **Redundancy** | Automatically falls back to Google on failure | Built-in fallback engine |
| **Status Badge** | `DeepL ⚡` (Cyan) | `Google` (Muted) / `Google 🔄` (Amber Fallback) |

</div>

<details>
<summary><b>🔑 How to get and set up your Free DeepL API Key (Click to expand)</b></summary>
<br>

1. Register for a free DeepL API account at [deepl.com/pro-api](https://www.deepl.com/pro-api) (Free plan provides 500,000 characters every month; Pro plans offer larger/unlimited tiers).
2. Copy your **Authentication Key** (Free keys end with `:fx`, e.g., `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx:fx`).
3. Open Louise Settings (`⚙`) → **Engine & API** tab.
4. Click the **DeepL API** card and paste your key into the text field.
5. The badge will instantly turn green (`Active ✓`) and your remaining quota will appear in real time!

</details>

---

## ⌨️ Shortcuts Cheatsheet

<div align="center">

| Key Combination | Action | Scope | Description |
| :---: | :--- | :---: | :--- |
| <kbd>Alt</kbd> + <kbd>C</kbd> | **Translate Selection** | Global | Primary hotkey; captures highlighted text and opens Louise near mouse |
| <kbd>Alt</kbd> + <kbd>S</kbd> | **Screen OCR Snip** | Global | Freezes screen to snip and extract text from images, videos, games, locked PDFs |
| <kbd>Alt</kbd> + <kbd>X</kbd> | **Toggle Window** | Global | Opens or closes Louise near cursor for direct typing (instant toggle) |
| <kbd>Ctrl</kbd> + <kbd>Enter</kbd> | **Translate Input** | In-App | Instantly triggers translation for manually typed source text |
| <kbd>Ctrl</kbd> + <kbd>C</kbd> / <kbd>V</kbd> / <kbd>A</kbd> / <kbd>X</kbd> | **Clipboard Actions** | In-App | Full bidirectional copy/paste working in both English and Ukrainian layouts |
| <kbd>Escape</kbd> | **Dismiss to Tray / Cancel Snip** | In-App / OCR | Silently hides Louise window back to tray, or cancels active screen snipping |
| <kbd>Right Click</kbd> | **Cancel Snip / Context Menu** | In-App / OCR | Right-click cancels snip instantly; inside app opens Copy/Paste/Clear context menu |

</div>

<details>
<summary><b>🛠️ How to record a Custom Key Combination (Click to expand)</b></summary>
<br>

1. Open Louise Settings (`⚙`) → **Shortcuts** tab.
2. Choose either **Translate Selection** or **Open Window**.
3. Click the recording field and hold down your desired combination (e.g. `Ctrl + Shift + T` or `Alt + Z`). The UI dynamically displays held keys in real-time.
4. Releasing the keys immediately commits the new combination and suspends old defaults.

</details>

---

## 🛠️ System Architecture

```mermaid
flowchart LR
    subgraph Desktop ["🖥️ Windows Desktop"]
        Hotkey["⌨️ Global Hotkeys\n(Alt+C / Alt+X)"]
        Selection["📄 Selected Foreign Text"]
        TargetApp["🎯 Active Application\n(IDE, Browser, Chat)"]
    end

    subgraph Louise ["🌌 Louise Core (Rust + Slint)"]
        UI["✨ Slint Declarative GUI\n(180Hz+ JetBrains Dark)"]
        Router{"🧠 Dual-Engine\nRouter"}
        Injector["🔄 Win32 Text Injector\n(In-Place Replace)"]
    end

    subgraph Cloud ["☁️ Neural Translation Providers"]
        DeepL["⚡ DeepL API\n(Neural Context Engine)"]
        Google["🌐 Google Translate\n(Auto Fallback)"]
    end

    Hotkey -->|Capture Selection| Louise
    Selection --> UI
    UI --> Router
    Router -->|Primary Choice| DeepL
    DeepL -.->|On Timeout or Limit| Google
    DeepL --> UI
    Google --> UI
    UI --> Injector
    Injector -->|Simulate Ctrl+V| TargetApp
```

- **GUI Toolkit**: [Slint](https://slint.dev/) (native compiled lightweight GUI)
- **Language**: [Rust](https://www.rust-lang.org/) (2024 Edition, zero runtime overhead)
- **OS Subsystem**: `windows-rs` (Win32 API, DWM frame controls, SendInput injection, DPAPI)
- **Global Hotkeys**: `global-hotkey`
- **System Tray**: `tray-icon`
- **Clipboard Management**: `arboard`

<p align="center">
  <img src="assets/divider.svg" width="60%" />
</p>

---

## 🚀 Quick Start & Build

### Option 1: Run Release Executable
1. Download `Louise Translator.exe` from [Latest Releases](https://github.com/MarianQQQ/LouiseTranslator/releases).
2. Run `Louise Translator.exe` (it will start discreetly in your system tray).
3. Highlight any text and press `Alt + C`.

### Option 2: Build from Source

#### Prerequisites
* [Rust Toolchain](https://rustup.rs/) (1.85+ recommended)
* Windows 10 or Windows 11
* Visual Studio C++ Build Tools

#### Compilation
```powershell
# 1. Clone the repository
git clone https://github.com/MarianQQQ/LouiseTranslator.git
cd LouiseTranslator

# 2. Compile optimized release build
cargo build --release

# 3. Executable will be located at:
# target\release\LouiseTranslator.exe
```

---

## 📂 Configuration

Preferences are stored locally in `config.json` alongside the executable:

```json
{
  "popup_mode": true,
  "always_on_top": true,
  "auto_copy": false,
  "close_on_paste": false,
  "src_lang": "auto",
  "dst_lang": "uk",
  "priority_langs": ["uk", "en", "pl"],
  "ui_lang": "en",
  "enable_alt_c": true,
  "use_deepl": true,
  "deepl_key": "YOUR_KEY_HERE:fx",
  "autostart": true
}
```

> [!NOTE]
> `config.json` is protected by `.gitignore` — your personal keys and preferences will never be tracked or committed.

---

<div align="center">

If Louise makes your multilingual workflow calmer and faster, consider giving the repository a star ⭐!

[![GitHub stars](https://img.shields.io/github/stars/MarianQQQ/LouiseTranslator?style=for-the-badge&logo=github&color=00C0F0)](https://github.com/MarianQQQ/LouiseTranslator/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/MarianQQQ/LouiseTranslator?style=for-the-badge&logo=github&color=0078D6)](https://github.com/MarianQQQ/LouiseTranslator/network/members)
[![GitHub issues](https://img.shields.io/github/issues/MarianQQQ/LouiseTranslator?style=for-the-badge&logo=github&color=22c55e)](https://github.com/MarianQQQ/LouiseTranslator/issues)

</div>

---

## 📜 Credits & Acknowledgments

- **Denis Villeneuve & Ted Chiang**: For the timeless world, linguistic depth, and soul of *Arrival*.
- **The Slint Team**: For creating the most responsive, elegant modern GUI toolkit.
- **DeepL & Google**: For providing world-class translation APIs.

---

## 📄 License

Distributed under the [MIT License](LICENSE) © 2026.
