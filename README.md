<div align="center">

<p align="center">
  <img src="assets/heptapod_circle.svg" width="160" height="160" alt="Rotating Heptapod B Logogram" />
</p>

# 🌌 Louise (Dr. Louise Banks)
### Discreet, Keyboard-First Instant Desktop Translator for Windows

<p align="center">
  <a href="https://github.com/MarianQQQ/LouiseTranslator">
    <img src="https://readme-typing-svg.demolab.com?font=JetBrains+Mono&weight=600&size=16&duration=3000&pause=1000&color=00C0F0&center=true&vCenter=true&width=620&lines=Discreet%2C+keyboard-first+instant+desktop+translator.;Highlight+text+%2B+Alt%2BC+%E2%86%92+Instant+translation.;Screen+OCR+%2B+Alt%2BS+%E2%86%92+Snip+any+image%2C+video%2C+PDF.;Dual-Engine%3A+DeepL+Neural+AI+%2B+Google+Fallback.;Neural+TTS+%28Edge+%2B+Google%2C+29+languages%29.;Crafted+with+pure+Rust+%26+Slint+UI+(~60+MB+RAM)." alt="Louise Typing Animation" />
  </a>
</p>

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Slint UI](https://img.shields.io/badge/Slint_UI-v1.18-00C0F0.svg?style=for-the-badge&logo=qt)](https://slint.dev/)
[![Windows](https://img.shields.io/badge/Platform-Windows_10_%7C_11-0078D6.svg?style=for-the-badge&logo=windows)](https://microsoft.com)
[![DeepL](https://img.shields.io/badge/DeepL-Neural_AI_Supported-00b4d8.svg?style=for-the-badge)](https://www.deepl.com/)
[![Windows OCR](https://img.shields.io/badge/Windows_OCR-Offline_%26_Native-7928ca.svg?style=for-the-badge&logo=windows)](https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr)
[![License](https://img.shields.io/badge/License-MIT-22c55e.svg?style=for-the-badge)](LICENSE)

<br>

> *«Language is the first weapon drawn in a conflict... or the first hand extended in understanding.»*  
> — **Dr. Louise Banks**, *Arrival* (2016)

</div>

---

## 📸 Interface Showcase

<p align="center">
  <img src="assets/ui_main.png" alt="Louise Main Translation Window" width="47%" style="border-radius: 8px;" />
  &nbsp;&nbsp;
  <img src="assets/ui_settings.png" alt="Louise Settings" width="47%" style="border-radius: 8px;" />
</p>
<p align="center">
  <i>Left: Main UI — Auto-detect, DeepL ⚡ badge, OCR button, inline TTS playback. Right: Engine & API settings with TTS provider selector.</i>
</p>

---

## ✨ Key Features

| | Feature |
|---|---|
| **⚡ Alt+C** | Highlight any text → instant translation appears next to cursor |
| **📷 Alt+S** | Freeze screen → drag to snip → OCR extracts & translates (100% offline, Windows Media OCR) |
| **🔊 TTS** | Neural voices via Edge TTS (29 langs) + Google TTS mutual fallback, switch in Settings |
| **🪟 Alt+X** | Toggle Louise window near cursor for manual input |
| **🔄 Replace** | Paste translated text directly back into any app with one click |
| **🧠 Dual-Engine** | DeepL for supported languages, auto-switches to Google for the rest — no setup needed |
| **🛡️ Fallback** | Auto-fallback on quota exhaustion, network errors, or unsupported language pairs |
| **🔒 Privacy** | Zero telemetry, DPAPI-encrypted keys, fully local `config.json` |
| **🚀 Native** | Rust + Slint — no Electron, ~60 MB RAM, sub-15ms start |

---

## 🧠 Dual-Engine Translation

<div align="center">

| | ⚡ DeepL API | 🌐 Google Translate |
| :--- | :--- | :--- |
| **Quality** | Neural AI, natural phrasing | Fast, reliable |
| **Setup** | Free or Pro API key | None — zero setup |
| **Auto-switch** | Used when language is supported | Fallback for the rest |
| **Badge** | `DeepL ⚡` | `Google` |

</div>

<details>
<summary><b>🔑 Free DeepL API Key setup</b></summary>
<br>

1. Register at [deepl.com/pro-api](https://www.deepl.com/pro-api) (Free: 500k chars/month).
2. Copy your **Authentication Key** (ends with `:fx`).
3. Open Louise → `⚙` Settings → **Engine & API** → paste key into DeepL card.

</details>

---

## 🔊 Neural Text-to-Speech

<div align="center">

| | 🎙️ Edge Neural TTS | 🔊 Google TTS |
| :--- | :--- | :--- |
| **Quality** | Natural Microsoft neural voices | Clear synthetic voices |
| **Languages** | 29 languages | All supported languages |
| **Fallback** | → Google on failure | → Edge on failure |

</div>

Switch providers: Settings → **Engine & API** → TTS section. Audio is synthesized with natural pacing (+3%) and cached per-provider for instant replay.

---

## 📷 Screen OCR — Windows Language Packs

Louise uses native **Windows.Media.Ocr** — offline, zero cloud tokens. Install OCR packs via PowerShell (as Administrator):

```powershell
# Install popular languages in one command:
@("uk-UA","en-US","pl-PL","de-DE","fr-FR","es-ES","it-IT","ja-JP") |
  ForEach-Object { Add-WindowsCapability -Online -Name "Language.OCR~~~$_~0.0.1.0" }
```

> **GUI alternative**: Win+I → Time & Language → Language & region → `...` → Language options → Download **OCR**.

---

## ⌨️ Shortcuts

<div align="center">

| Shortcut | Action |
| :---: | :--- |
| <kbd>Alt</kbd>+<kbd>C</kbd> | Translate selected text |
| <kbd>Alt</kbd>+<kbd>S</kbd> | Screen OCR snip |
| <kbd>Alt</kbd>+<kbd>X</kbd> | Toggle window |
| <kbd>Ctrl</kbd>+<kbd>Enter</kbd> | Translate typed input |
| <kbd>Escape</kbd> | Hide to tray / cancel snip |

</div>

All shortcuts are customizable in Settings → **Shortcuts** tab.

---

## 🚀 Quick Start

**Option 1 — Download**: Grab `Louise Translator.exe` from [Releases](https://github.com/MarianQQQ/LouiseTranslator/releases) and run it.

**Option 2 — Build from source**:
```powershell
git clone https://github.com/MarianQQQ/LouiseTranslator.git
cd LouiseTranslator
cargo build --release
```

---

## 📂 Configuration

`config.json` is stored locally next to the executable (never tracked by Git, keys encrypted via DPAPI):

```json
{
  "src_lang": "auto",
  "dst_lang": "uk",
  "priority_langs": ["uk", "en", "pl"],
  "use_deepl": true,
  "deepl_key": "YOUR_KEY_HERE:fx",
  "use_edge_tts": true,
  "autostart": true
}
```

---

<div align="center">

If Louise makes your multilingual workflow calmer and faster, consider giving the repo a ⭐!

[![GitHub stars](https://img.shields.io/github/stars/MarianQQQ/LouiseTranslator?style=for-the-badge&logo=github&color=00C0F0)](https://github.com/MarianQQQ/LouiseTranslator/stargazers)

*Inspired by Denis Villeneuve's «Arrival» · Built with [Rust](https://www.rust-lang.org/) & [Slint](https://slint.dev/) · [MIT License](LICENSE) © 2026*

</div>
