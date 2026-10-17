# Контракти поведінки та специфікація Louise Translator (Android)

> Згідно з Маніфестом міграції (Фаза 2: `02-contract.md`).  
> Мета: зафіксувати точні входи, виходи, інваріанти та протоколи поведінки.

---

## 1. Контракт точки входу: Системне виділення тексту (`ACTION_PROCESS_TEXT`)

Коли користувач виділяє текст у будь-якій програмі:

- **Вхідний Intent**:
  - `action`: `android.intent.action.PROCESS_TEXT`
  - `type`: `text/plain`
  - `extra`: `EXTRA_PROCESS_TEXT` (рядок `CharSequence`, що містить виділений фрагмент)
  - `extra`: `EXTRA_PROCESS_TEXT_READONLY` (булеве значення: `true` якщо поле доступне лише для читання, `false` якщо підтримує заміну тексту)
- **Поведінка**:
  1. Отримати рядок із `EXTRA_PROCESS_TEXT`.
  2. Обрізати зайві пробіли на краях (`trim()`). Якщо рядок порожній — відкрити інтерфейс для ручного вводу.
  3. Відкрити вікно перекладу (Floating Overlay або BottomSheet) поверх поточної програми.
  4. Ініціювати переклад за правилами `UC-04`.
- **Контракт "Paste as replacement" (Заміна у вихідному полі)**:
  - Якщо `EXTRA_PROCESS_TEXT_READONLY == false` і користувач натискає «Paste as replacement»:
    ```kotlin
    val intent = Intent().apply {
        putExtra(Intent.EXTRA_PROCESS_TEXT, translatedText)
    }
    setResult(Activity.RESULT_OK, intent)
    finish()
    ```
  - Якщо поле `READONLY == true`: кнопка «Paste as replacement» замінюється або діє як «Copy to clipboard» із показом спливаючого повідомлення «Copied! ✓».

---

## 2. Контракт перекладу (`TranslateEngineContract`)

### 2.1. Вхідні дані:
```
translate(
    text: String,
    srcLang: String,       // "auto" або ISO-код ("en", "uk", "de", "pl"...)
    dstLang: String,       // ISO-код ("uk", "en", "de"...)
    deepLKey: String?,     // Опціональний ключ DeepL
    useDeepL: Boolean      // Прапорець використання DeepL
): Result<TranslationResult>
```

### 2.2. Вихідні дані (`TranslationResult`):
```
struct TranslationResult {
    translatedText: String,
    detectedLang: String?,    // Реальний ISO код мови (якщо srcLang == "auto")
    engineUsed: EngineType,   // DEEPL або GOOGLE
    fellBack: Boolean         // true, якщо DeepL зазнав збою і було застосовано Google
}
```

### 2.3. Алгоритм та інваріанти:
1. **Порожній рядок**: якщо `text.trim().isEmpty()`, переклад не виконується, повертається порожній результат, лічильник символів скидається в `0`.
2. **Debounce**: тайм-аут 400 мс при наборі тексту. Будь-який новий символ скасовує попередній запит у польоті.
3. **DeepL умови**:
   - `useDeepL == true` ТА `deepLKey != null && !deepLKey.isBlank()` ТА пара мов `(src, dst)` підтримується DeepL.
   - Endpoint: `https://api-free.deepl.com/v2/translate` (якщо ключ закінчується на `:fx`) або `https://api.deepl.com/v2/translate`.
   - Заголовки: `Authorization: DeepL-Auth-Key <key>`, `Content-Type: application/json`.
4. **Google Fallback**:
   - Викликається, якщо: DeepL вимкнено, ключа немає, пара мов не підтримується DeepL, або запит до DeepL повернув HTTP 4xx/5xx чи мережеву помилку.
   - Endpoint: `https://translate.googleapis.com/translate_a/single?client=gtx&sl=<src>&tl=<dst>&dt=t&q=<encoded_text>`.
5. **Автовизначення мови**:
   - Якщо мова джерела `"auto"`:
     - Спочатку евристичний швидкий аналіз алфавіту (`Latin` vs `Cyrillic`): якщо кирилиця → цільова мова за замовчуванням `en`, якщо латиниця → цільова мова `uk`.
     - З відповіді Google Translate / DeepL витягується точний `detected_source_language` (наприклад, `"de"`).
     - Цей код фіксується у `detected_src_lang_code` для подальшого озвучення вихідного тексту!

---

## 3. Контракт озвучення (`TtsContract`)

1. **Керування програвачем**:
   - Запит на озвучення джерела (`play_src_tts(text, lang)`) ПОВИНЕН зупинити озвучення перекладу, якщо воно грає.
   - Запит на озвучення перекладу (`play_dst_tts(text, lang)`) ПОВИНЕН зупинити озвучення джерела.
   - Повторний клік на кнопку в активному стані (`Stop`) ПОВИНЕН зупинити поточне аудіо.
2. **Вибір мови**:
   - Для цільового тексту: використовується обраний `dst_lang`.
   - Для вихідного тексту: якщо `src_lang != "auto"` → `src_lang`. Якщо `src_lang == "auto"` → береться визначений `detected_src_lang_code`. Якщо мову ще не визначено → за замовчуванням `en`.
3. **Джерела звуку на Android**:
   - **Пріоритет 1 (Локальний)**: `android.speech.tts.TextToSpeech` (вбудовані нейронні моделі Google, 0 трафіку, миттєвий старт).
   - **Пріоритет 2 (Хмарний / Edge)**: Microsoft Edge Neural TTS через існуючий протокол (якщо обрано в налаштуваннях).

---

## 4. Контракт локального сховища (`ConfigContract`)

Зберігання ключів та налаштувань:
- Ключі DeepL: **НІКОЛИ** не зберігати у звичайному `SharedPreferences` чи відкритому JSON-файлі. Використовувати `MasterKeys` / `EncryptedSharedPreferences` (апаратний захист AES-256 через Android Keystore).
- Схема налаштувань (ідентична до Windows `config.json`):
  - `src_lang: String` (за замовчуванням `"auto"`)
  - `dst_lang: String` (за замовчуванням `"uk"`)
  - `priority_langs: List<String>` (набір вибраних мов швидкого доступу)
  - `use_deepl: Boolean` (за замовчуванням `true`)
  - `deepl_key: String` (зашифровано)
  - `use_edge_tts: Boolean` (за замовчуванням `true`)
  - `auto_copy: Boolean` (за замовчуванням `false`)
  - `close_on_paste: Boolean` (за замовчуванням `true`)
