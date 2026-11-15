# ForgeDesk — пакет проекта и технического задания

**Версия пакета:** 1.0-draft  
**Дата подготовки:** 15 ноября 2026 года  
**Статус:** утверждённая продуктовая концепция; техническое задание подготовлено для передачи DeepSeek API  
**Рабочее имя:** ForgeDesk (может быть изменено без изменения архитектуры)

> **Implementation status (2026-11-15):** the Stage 1 desktop foundation is
> implemented in this repository. See `docs/PROJECT_STATE.md` for current state,
> `docs/development/setup.md` for commands and `docs/ui/UI-GATE-0-REPORT.md` and
> `docs/ui/UI-GATE-1-REPORT.md` for visual acceptance evidence. The requirement documents below remain authoritative.

## Назначение пакета

Этот пакет описывает создание локального desktop-приложения для индивидуального управления разработкой. По охвату управления задачами и проектами оно должно приблизиться к полной функциональности Linear, но не копировать его многопользовательскую SaaS-архитектуру. Главные отличия — локальная база SQLite, глубокая интеграция GitHub, экономная и безопасная синхронизация API, адаптеры для GitLab/Gitea и полностью необязательный AI-модуль.

Пакет рассчитан на передачу в DeepSeek API как самодостаточный контекст разработки. Репозиторий приложения пока не предоставлен, поэтому структура каталогов в `02_TECHNICAL_SPEC.md` является целевой, а не описанием уже существующего кода. После создания/выбора репозитория DeepSeek должен сначала провести короткую инвентаризацию, сопоставить целевую структуру с фактической и зафиксировать отличия — без серии вопросов по каждому файлу.

## Состав

| Файл | Назначение |
|---|---|
| `01_PRODUCT_PROJECT.md` | Проект, продуктовая концепция, функциональный охват, сценарии и UX |
| `02_TECHNICAL_SPEC.md` | Основное ТЗ: архитектура, доменная модель, модули, этапы, критерии приёмки и тестирование |
| `03_GITHUB_API_BUDGET_AND_SYNC.md` | Подробное ТЗ для GitHub GraphQL/REST, rate limits, бюджетирования и синхронизации |
| `04_TOKEN_EFFICIENCY_RULES.md` | Обязательные требования к экономии токенов DeepSeek API и работе с контекстом/репозиторием |
| `05_DEEPSEEK_PROMPTS.md` | Warm prompt, starter prompt и шаблон промпта для следующих этапов |
| `06_UI_UX_SPEC.md` | Обязательный UI/UX-контракт: точная геометрия, дизайн-токены, экраны, состояния, клавиатура и screenshot acceptance |

## Как передавать пакет DeepSeek

1. Передайте все шесть документов `01_PRODUCT_PROJECT.md`–`06_UI_UX_SPEC.md` как проектный контекст. `06_UI_UX_SPEC.md` обязателен: без него нельзя начинать frontend-разработку.
2. Вставьте warm prompt из `05_DEEPSEEK_PROMPTS.md`.
3. После того как DeepSeek кратко зафиксирует архитектуру, риски и план первого вертикального этапа, передайте starter prompt из того же файла.
4. Не просите модель реализовать весь продукт одним ответом. Реализация ведётся по вертикальным этапам с локальными тестами и небольшими diff.
5. После появления репозитория добавьте его URL/архив. Только тогда можно требовать точные изменения существующих файлов.
6. На Stage 1 потребуйте завершить UI Gate 0 из `06_UI_UX_SPEC.md`; не принимайте пустой generic shell. До реализации функциональных экранов должны существовать и быть проверены design tokens, AppShell, navigation, themes и общие компоненты.

## Обязательные продуктовые ограничения

- Продукт — одно локальное desktop-приложение, Windows и Linux в первой целевой версии.
- Нет собственного backend-сервера, облачной БД, обязательного Docker, Redis, PostgreSQL, Nginx или Tomcat.
- Основная база — SQLite; приложение работает без интернета.
- GitHub — главный внешний Git-провайдер; GraphQL предпочтителен, REST применяется там, где он нужен/лучше поддержан.
- API rate-limit manager и sync scheduler — обязательные компоненты ядра, не второстепенная оптимизация.
- AI — полностью необязательное расширение. Core не зависит от AI SDK, API-ключей, фоновых AI-процессов или MCP.
- UI не оставлен на усмотрение DeepSeek: `06_UI_UX_SPEC.md` — обязательный визуальный контракт, включая screen layouts, dimensions, tokens, components, states and screenshot review.
- Операции записи в GitHub не должны автоматически повторяться при неопределённом результате. Сначала выполняется сверка удалённого состояния, чтобы не создавать дубликаты.
- Локальные данные и локальные задачи не публикуются в GitHub без явного пользовательского действия или заранее созданного пользователем правила.

## Терминология

- **Local Issue** — задача, существующая только в ForgeDesk.
- **Linked Issue** — локальная задача, связанная с GitHub Issue или Issue другого провайдера.
- **Remote entity** — внешний объект Git-провайдера.
- **Provider adapter** — адаптер API конкретного Git-сервиса.
- **Sync** — обмен изменениями между локальной БД и удалённым провайдером.
- **API budget** — внутренний защитный бюджет запросов, рассчитанный по последним фактическим данным провайдера.
- **AI Extension** — отключаемый модуль, не входящий в обязательный runtime Core.

## Источники технической проверки

Использовать официальную документацию как первичный источник, проверять её повторно непосредственно перед релизом: ограничения GitHub могут меняться.

- GitHub GraphQL rate/query limits: https://docs.github.com/en/graphql/overview/rate-limits-and-query-limits-for-the-graphql-api
- GitHub REST rate limits: https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api
- GitHub REST best practices: https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api
- GitHub App authentication: https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app
- GitHub App user access token / Device Flow: https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app
- GitHub App best practices: https://docs.github.com/en/apps/creating-github-apps/about-creating-github-apps/best-practices-for-creating-a-github-app
- DeepSeek context caching: https://api-docs.deepseek.com/guides/kv_cache/
- DeepSeek API usage/pricing: https://api-docs.deepseek.com/quick_start/pricing/
- Tauri v2: https://v2.tauri.app/
- Tauri SQL plugin security model (reference only; ForgeDesk DB access should stay behind Rust repositories): https://v2.tauri.app/plugin/sql/

> Значения лимитов в ТЗ приведены как актуальные ориентиры документации на дату подготовки. Реализация должна использовать фактические headers/API responses и не зашивать предположение о неизменности квот.
