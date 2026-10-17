# Інвентаризація Louise Translator (Windows → Android)

> Згідно з Маніфестом міграції (Фаза 1: `01-inventory.md`).  
> Мета: повна фіксація поведінки, модулів, залежностей та інваріантів перед початком переносу на Android.

---

## 1. Сценарії використання (User Stories & Use Cases)

| ID | Сценарій на Windows | Аналог на Android | Джерело в коді |
| :--- | :--- | :--- | :--- |
| **UC-01** | **Alt+C**: Виділення тексту в будь-якому вікні → копіювання в буфер → відкриття вікна перекладу. | **ACTION_PROCESS_TEXT**: Виділення тексту в Chrome/Telegram/нотатках → системне меню «Louise» → відкриття плаваючого вікна / боттом-шіта. | `src/main.rs:608-688`, `src/hotkeys.rs` |
| **UC-02** | **Alt+X**: Швидкий виклик вікна перекладача для ручного вводу тексту біля курсора. | Відкриття застосунку з лаунчера або через Quick Settings Tile (шторка сповіщень) / плаваючу бульбашку (bubble). | `src/main.rs:690-739` |
| **UC-03** | **Alt+S**: Заморозка екрана, напівпрозоре виділення області мишкою (Screen OCR Snip) → розпізнавання тексту. | **MediaProjection / Screen Capture** або виділення на екрані + **Google ML Kit Text Recognition** (локально, офлайн). | `src/snipping.rs`, `src/ocr.rs` |
| **UC-04** | **Dual-Engine переклад**: Ввід тексту → 400мс debounce → переклад через DeepL (якщо мова підтримується і є ключ) з авто-фолбеком на Google Translate. | Ідентична бізнес-логіка: DeepL API v2 + Google Translate (gapi/single gtx fallback) + 400мс debounce. | `src/translate.rs`, `src/main.rs:767-865` |
| **UC-05** | **Автовизначення мови**: При `src_lang == "auto"` визначається мова вводу, оновлюється бейдж мови та код мови для вимови. | Ідентично (`auto_detect_target_lang` + визначення мови з відповіді API). | `src/translate.rs`, `src/main.rs:135-147` |
| **UC-06** | **Dual TTS (Озвучення джерела й перекладу)**: Кнопки `Listen` у заголовках SOURCE та TRANSLATION з перемиканням на `Stop`. Автозупинка протилежного програвача. | **Android TextToSpeech API** або Microsoft Edge Neural TTS через WebSocket/HTTP. Взаємна зупинка потоків аудіо. | `src/tts.rs`, `src/main.rs:1410-1510` |
| **UC-07** | **Заміна виділеного тексту ("Paste as replacement")**: Натискання кнопки вставляє переклад назад у вихідне поле попередньої програми. | Повернення результату через `setResult(RESULT_OK, intent)` для `ACTION_PROCESS_TEXT` (система сама автоматично замінює виділений фрагмент у полі вводу!). | `src/main.rs:1340-1360`, `src/win32.rs` |
| **UC-08** | **Налаштування та пріоритетні мови**: Вибір вихідної/цільової мов, збереження DeepL ключа, вибір рушія TTS, налаштування поведінки. | Екран налаштувань (Settings), локальне захищене сховище (EncryptedSharedPreferences / Android Keystore). | `src/config.rs`, `ui/appwindow.slint` |

---

## 2. Аналіз модулів та матриця перенесення коду

| Файл у Windows | Призначення | Платформозалежність | Рішення для Android |
| :--- | :--- | :--- | :--- |
| `src/translate.rs` | Запити до Google Translate та DeepL, парсинг відповідей, мапінг мов | **Чиста платформа (Pure Rust)** — `reqwest`, `serde`, `tokio` | **100% перевикористання** у спільному Rust-ядрі або ідіоматичний порт на Kotlin (Ktor/OkHttp). |
| `src/tts.rs` | Edge Neural TTS (WebSocket), Google TTS fallback, розбиття тексту, кешування аудіо | **Частково платформозалежний**: аудіо-генерація чиста, але програвання через `windows::Media::Playback::MediaPlayer` | Генерація звуку лишається спільною; програвання делегується нативному `android.media.MediaPlayer` або Android TTS. |
| `src/config.rs` | Збереження налаштувань, шифрування ключів | **Платформозалежний**: Windows DPAPI (`CryptProtectData`) | Заміна на Android Keystore / `EncryptedSharedPreferences`. Контракт полів `AppConfig` зберігається 1:1. |
| `src/ocr.rs` | Обробка бітових карт, виклик `Windows.Media.Ocr.OcrEngine` | **100% Windows WinRT** | Заміна на **Google ML Kit Text Recognition** (локальна модель, швидка, підтримує Latin/Cyrillic/CJK/Devanagari, 0 хмарних токенів). |
| `src/snipping.rs` | GDI/DWM захоплення екрана, прозоре вікно поверх моніторів | **100% Windows Win32** | Заміна на Android `MediaProjection` API + накладання прозорого `Overlay View`. |
| `src/hotkeys.rs` | Глобальні клавіатурні комбінації `RegisterHotKey` (Alt+C, Alt+X, Alt+S) | **100% Windows Win32** | На Android замінюється на реєстрацію `ACTION_PROCESS_TEXT` у системному маніфесті + Quick Settings Tile / Bubble. |
| `src/win32.rs` | Маніпуляції вікнами HWND, DWM тіні, симуляція натискання Ctrl+V | **100% Windows Win32** | На Android відсутні HWND; використовуються `WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY`. |
| `ui/appwindow.slint` | Дизайн інтерфейсу, теми, іконки, діалоги вибору мов | **Slint UI** (підтримує Android через NDK) | Може збиратися через Slint Android backend, або відтворюватися нативно через **Jetpack Compose** зі збереженням шрифтів JetBrains Mono, кольорів `#101010` та геометрії. |

---

## 3. Зовнішні залежності та їхні аналоги

| Залежність Rust (Windows) | Що робить | Цільовий аналог на Android |
| :--- | :--- | :--- |
| `reqwest` + `tokio` | Async HTTP/HTTPS для API перекладу | `reqwest` (якщо Rust NDK) або `OkHttp` / `Ktor-client` (якщо Kotlin) |
| `kothok-edge-tts` / `tungstenite` | WebSocket з'єднання з Edge Neural TTS | Нативний виклик того ж endpoint або `android.speech.tts.TextToSpeech` |
| `windows` (WinRT OCR) | Офлайн розпізнавання тексту | `com.google.mlkit:text-recognition` (офлайн на пристрої) |
| `dpapi` (Windows Crypto) | Захист DeepL API ключа | `androidx.security:security-crypto` (Android Keystore + AES-256 GCM) |
| `arboard` / `clipboard-win` | Буфер обміну ОС | `android.content.ClipboardManager` |
| `tray-icon` / `muda` | Системний трей і контекстне меню | Android Notification з діями та Quick Settings Tile |

---

## 4. Нефункціональні вимоги та інваріанти

1. **Швидкість відгуку (Latency)**: відкриття вікна перекладу < 150мс після натискання «Louise» у меню виділення тексту.
2. **Пам'ять (RAM)**: не більше 40–60 МБ у фоновому стані (на рівні поточної версії для ПК).
3. **Офлайн-приватність**: Жодної аналітики, телеметрії, сторонніх рекламних SDK. Збереження API ключів виключно в апаратно захищеному сховищі телефону.
4. **Кольорова палітра та стиль**:
   - Фон вікна: `#101010`
   - Фон карток та інпутів: `#1c1c1c`
   - Бордери: `#282828` / Акцент фокусу: `#f1ffff` (або фірмовий ціан `#00c0f0`)
   - Текст: `#f1ffff` та `#a1a1aa`
   - Шрифт: `JetBrains Mono` для інпутів і бейджів.
