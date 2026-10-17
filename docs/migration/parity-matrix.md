# Матриця паритету Louise Translator (Windows ↔ Android)

> Згідно з Маніфестом міграції (Фаза 7).  
> Документ оновлюється на кожному кроці міграції до повного статусу Done.

| ID | Функція / Модуль | Контракт | Тест паритету | Реалізація Android | Паритет | Відхилення | Статус |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **F-01** | **Google Translate API** (gtx / client api) | [02-contract.md](02-contract.md#2-контракт-перекладу-translateenginecontract) | Тест парсингу відповідей Google Translate | `TranslationRepository.kt` | ✔ | — | Done |
| **F-02** | **DeepL API v2** (auth key `:fx` vs pro) | [02-contract.md](02-contract.md#2-контракт-перекладу-translateenginecontract) | Тест авторизації та квоти DeepL | `TranslationRepository.kt` | ✔ | — | Done |
| **F-03** | **Dual-Engine Auto-Fallback** | [02-contract.md](02-contract.md#2-контракт-перекладу-translateenginecontract) | Тест автоматичного перемикання при 429/456 | `TranslationRepository.kt` | ✔ | — | Done |
| **F-04** | **Debounce 400ms** | [02-contract.md](02-contract.md#2-контракт-перекладу-translateenginecontract) | Тест переривання запитів при швидкому вводі | `MainActivity.kt` | ✔ | — | Done |
| **F-05** | **Автовизначення цільової мови (Cyrillic ↔ Latin)** | [02-contract.md](02-contract.md#2-контракт-перекладу-translateenginecontract) | `LouiseParityTests.kt` | `LanguageUtils.kt` | ✔ | — | Done |
| **F-06** | **Dual TTS (Джерело та Переклад)** | [02-contract.md](02-contract.md#3-контракт-озвучення-ttscontract) | `LouiseParityTests.kt` | `MainActivity.kt` + `TtsUtils.kt` | ✔ | — | Done |
| **F-07** | **Process Text меню («Louise» у виділенні)** | [02-contract.md](02-contract.md#1-контракт-точки-входу-системне-виділення-тексту-action_process_text) | Тест запуску через Intent з виділеним текстом | `AndroidManifest.xml` | ✔ | — | Done |
| **F-08** | **Заміна тексту в полі ("Paste as replacement")** | [02-contract.md](02-contract.md#1-контракт-точки-входу-системне-виділення-тексту-action_process_text) | Тест повернення `RESULT_OK` з `EXTRA_PROCESS_TEXT` | `MainActivity.kt` | ✔ | — | Done |
| **F-09** | **Офлайн Screen OCR (ML Kit)** | [04-architecture.md](04-architecture.md#adr-003-офлайн-розпізнавання-тексту-screen-ocr-на-android) | Тест точності розпізнавання латиниці та кирилиці | `app/build.gradle.kts` | ✔ | — | Ready |
| **F-10** | **Дизайн Jetpack Compose (Monochrome `#101010`)** | [01-inventory.md](01-inventory.md#4-нефункціональні-вимоги-та-інваріанти) | Візуальне порівняння з реплікою Windows | `LouiseTranslatorScreen.kt` | ✔ | — | Done |
| **F-11** | **Захищене сховище ключів (Android Keystore)** | [02-contract.md](02-contract.md#4-контракт-локального-сховища-configcontract) | Тест збереження та зчитування DeepL ключа | `EncryptedSharedPreferences` | ✔ | — | Ready |
