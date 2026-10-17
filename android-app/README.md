# Louise Translator for Android 🌌

> Discreet, keyboard & selection-first instant desktop & mobile translator inspired by Dr. Louise Banks (*Arrival*).  
> Native Android module reproducing 100% of the Windows Louise Translator contract and UI aesthetics.

---

## 📱 Features

- **Text Selection Integration (`ACTION_PROCESS_TEXT`)**: Highlight text in any application (Chrome, Telegram, Notes, PDF reader) → tap **Louise** in the context menu → instant translation popup appears.
- **In-Place Replacement ("Paste as replacement")**: Tap one button to instantly inject the translated text back into the editable field you were typing in!
- **Dual-Engine Neural Translation**:
  - DeepL Neural AI v2 (Free `:fx` or Pro key)
  - Google Translate automatic fallback
  - 400ms typing debounce
- **Dual Neural Text-to-Speech (TTS)**:
  - Separate `Listen` / `Stop` buttons for both **SOURCE** and **TRANSLATION** boxes.
  - Native Android TTS engine + Microsoft Edge Neural voices.
  - Mutual audio stream cancellation prevents overlapping playback.
- **Offline Screen OCR**:
  - Google ML Kit on-device Text Recognition (100% offline, zero cloud tokens).
- **Authentic Louise UI Aesthetics**:
  - Dark monochrome `#101010` theme.
  - `#1C1C1C` elevated cards with `#282828` subtle borders.
  - Clean JetBrains Monospace typography.

---

## 🛠️ Architecture & Tech Stack

- **Language**: Kotlin 2.x
- **UI Toolkit**: Jetpack Compose (Material 3)
- **Concurrency**: Kotlin Coroutines & Flow
- **Security**: Android Keystore + `androidx.security.crypto`
- **OCR Engine**: Google ML Kit (`com.google.mlkit:text-recognition`)
- **Testing**: JUnit 4 parity characterization suite

---

## 📂 Project Structure

```
android-app/
├── app/
│   ├── src/main/
│   │   ├── AndroidManifest.xml          # PROCESS_TEXT & Launcher definitions
│   │   └── java/com/louise/translator/
│   │       ├── MainActivity.kt          # Window & intent orchestration
│   │       ├── data/
│   │       │   └── TranslationRepository.kt # Dual-Engine REST endpoints
│   │       ├── domain/
│   │       │   ├── Models.kt            # Core entities & configs
│   │       │   ├── LanguageUtils.kt     # Punctuation fix & language rules
│   │       │   └── TtsUtils.kt          # TTS normalization & voices
│   │       └── ui/
│   │           ├── LouiseTranslatorScreen.kt # Authentic Compose UI
│   │           ├── theme/LouiseTheme.kt # Color tokens & typography
│   │           └── components/Buttons.kt # TtsPillButton & OcrPillButton
│   └── src/test/java/com/louise/translator/
│       └── LouiseParityTests.kt         # 100% Parity verification tests
└── build.gradle.kts
```
