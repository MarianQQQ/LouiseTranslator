# Матриця паритету Louise Translator (Windows ↔ Android)

> Згідно з Маніфестом міграції (Фаза 7).  
> Документ оновлюється на кожному кроці міграції до повного статусу Done.

| ID | Функція / Модуль | Контракт | Тест паритету | Реалізація Android | Паритет | Відхилення | Статус |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **F-01** | **Google Translate API** (gtx / client api) | [02-contract.md](02-contract.md#2-контракт-перекладу-translateenginecontract) | Тест парсингу відповідей Google Translate | В процесі | Очікується | — | In Progress |
| **F-02** | **DeepL API v2** (auth key `:fx` vs pro) | [02-contract.md](02-contract.md#2-контракт-перекладу-translateenginecontract) | Тест авторизації та квоти DeepL | В процесі | Очікується | — | In Progress |
| **F-03** | **Dual-Engine Auto-Fallback** | [02-contract.md](02-contract.md#2-контракт-перекладу-translateenginecontract) | Тест автоматичного перемикання при 429/456 | В процесі | Очікується | — | In Progress |
| **F-04** | **Debounce 400ms** | [02-contract.md](02-contract.md#2-контракт-перекладу-translateenginecontract) | Тест переривання запитів при швидкому вводі | В процесі | Очікується | — | Planned |
| **F-05** | **Автовизначення цільової мови (Cyrillic ↔ Latin)** | [02-contract.md](02-contract.md#2-контракт-перекладу-translateenginecontract) | `test_auto_detect_target_lang` | В процесі | Очікується | — | Planned |
| **F-06** | **Dual TTS (Джерело та Переклад)** | [02-contract.md](02-contract.md#3-контракт-озвучення-ttscontract) | Тест взаємного переривання аудіо | В процесі | Очікується | — | Planned |
| **F-07** | **Process Text меню («Louise» у виділенні)** | [02-contract.md](02-contract.md#1-контракт-точки-входу-системне-виділення-тексту-action_process_text) | Тест запуску через Intent з виділеним текстом | В процесі | Очікується | — | Planned |
| **F-08** | **Заміна тексту в полі ("Paste as replacement")** | [02-contract.md](02-contract.md#1-контракт-точки-входу-системне-виділення-тексту-action_process_text) | Тест повернення `RESULT_OK` з `EXTRA_PROCESS_TEXT` | В процесі | Очікується | — | Planned |
| **F-09** | **Офлайн Screen OCR (ML Kit)** | [04-architecture.md](04-architecture.md#adr-003-офлайн-розпізнавання-тексту-screen-ocr-на-android) | Тест точності розпізнавання латиниці та кирилиці | В процесі | Очікується | — | Planned |
| **F-10** | **Дизайн Jetpack Compose (Monochrome `#101010`)** | [01-inventory.md](01-inventory.md#4-нефункціональні-вимоги-та-інваріанти) | Візуальне порівняння з реплікою Windows | В процесі | Очікується | — | Planned |
| **F-11** | **Захищене сховище ключів (Android Keystore)** | [02-contract.md](02-contract.md#4-контракт-локального-сховища-configcontract) | Тест збереження та зчитування DeepL ключа | В процесі | Очікується | — | Planned |
