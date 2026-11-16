# 02. ForgeDesk — техническое задание

**Документ:** основной Software Requirements Specification (SRS)  
**Версия:** 1.0-draft  
**Связанные документы:** `01_PRODUCT_PROJECT.md`, `03_GITHUB_API_BUDGET_AND_SYNC.md`, `04_TOKEN_EFFICIENCY_RULES.md`, `06_UI_UX_SPEC.md`.

## 1. Цель разработки

Разработать самостоятельное локальное desktop-приложение для индивидуального управления жизненным циклом разработки. Приложение должно объединять управление задачами/проектами уровня Linear, локальный Git workflow и глубокую интеграцию GitHub с экономным синхронизатором.

ТЗ является целевым контрактом продукта. Реализацию выполнять постепенно. Нельзя выдавать за готовые функции, которые реализованы только визуально или возвращают тестовые данные.

## 2. Приоритет требований

- **P0 — обязательный фундамент:** приложение не может считаться работоспособным без этого.
- **P1 — обязательный функционал первой полноценной версии:** пользовательский базовый workflow.
- **P2 — расширенный функционал:** завершение охвата Linear.
- **P3 — опциональные расширения:** AI и дополнительные провайдеры.

Если требование не входит в текущий этап, оно всё равно учитывается в архитектуре, но не должно провоцировать преждевременную реализацию всех зависимых модулей.

## 3. Технологический стек и runtime

### 3.1. Целевой стек

- Desktop shell: Tauri 2.
- Backend/domain/core: Rust Stable.
- Frontend: Vue 3 + TypeScript + Vite.
- Database: SQLite (предпочтительно bundled SQLite; Rust доступ к БД через repository layer).
- FTS: SQLite FTS5.
- Git: системный `git` executable.
- GitHub: GraphQL основной, REST вспомогательный.
- GitLab: provider-specific GraphQL/REST.
- Gitea/Forgejo: REST.

Точные crates и frontend-библиотеки выбрать после проверки актуальных поддерживаемых версий и совместимости. Не вводить библиотеку без практической необходимости. Версии зависимостей фиксировать в lockfiles.

### 3.2. Запреты на архитектурном уровне

Не использовать как обязательные компоненты:
- отдельный HTTP backend server;
- PostgreSQL/MySQL;
- Redis;
- Docker/Kubernetes;
- Nginx/Tomcat;
- облачную БД;
- постоянный daemon/agent;
- Electron/встроенный отдельный Chromium runtime;
- обязательный AI SDK или AI API.

Tauri webview — только UI. Вся бизнес-логика, права на операции, API credentials, Git-процессы и доступ к SQLite находятся в Rust Core.

### 3.3. Предлагаемая структура проекта

Структура может быть адаптирована под фактический репозиторий, но логические границы должны сохраниться.

```text
forgedesk/
  README.md
  CONTRIBUTING.md
  CHANGELOG.md
  docs/
    architecture/
    decisions/
    development/
  src/
    app/
    components/
    composables/
    features/
      inbox/
      issues/
      projects/
      initiatives/
      cycles/
      documents/
      repositories/
      pull-requests/
      reviews/
      analytics/
      settings/
    router/
    stores/
    styles/
    types/
  src-tauri/
    Cargo.toml
    tauri.conf.json
    capabilities/
    src/
      main.rs
      app/
      commands/
      domain/
      db/
        migrations/
        repositories/
      providers/
        mod.rs
        github/
        gitlab/
        gitea/
      sync/
      api_budget/
      git/
      search/
      automation/
      backup/
      ai_extension/      # optional feature-gated crate/module
      error/
  tests/
    integration/
    fixtures/
    provider_contracts/
```

Не создавать монолитный `main.rs`, общий `utils.rs` на сотни строк или единый файл со всей бизнес-логикой. Модули должны оставаться компактными, с явными интерфейсами.

## 4. Архитектурные границы

### 4.1. Слои

1. **UI layer (Vue).** Отображение и пользовательское взаимодействие. Не обращается к SQLite или GitHub напрямую.
2. **Tauri command/application layer.** Типизированные команды, валидация входных аргументов, преобразование ошибок в стабильные UI errors.
3. **Domain layer.** Независимые модели, правила workflow, отношения задач, проекты, циклы, локальные автоматизации.
4. **Repository/DB layer.** Все SQL, миграции и транзакции.
5. **Provider layer.** Интерфейсы и адаптеры GitHub/GitLab/Gitea.
6. **Sync/API Budget layer.** Очередь, приоритеты, rate-limit accounting, cursors, retries, conflict handling.
7. **Local Git layer.** Работа с системным Git.
8. **Optional AI layer.** Отдельная сборочная возможность; не импортируется обязательными модулями.

### 4.2. Зависимости

- Domain не зависит от Tauri, Vue, GitHub SDK, AI SDK или HTTP-клиента.
- DB layer не содержит provider-specific бизнес-правил.
- Provider adapters возвращают provider-neutral DTO и capability metadata.
- Sync layer знает об API budgets, но бизнес-модули не должны самостоятельно отправлять HTTP-запросы.
- UI не может запускать произвольный SQL, shell-команды или HTTP.
- AI Extension использует публичные application services, но Core не зависит от AI Extension.

### 4.3. Модель команды приложения

Все команды Tauri — typed commands с DTO. Не экспортировать универсальный `execute_sql`, `run_shell`, `fetch_url` или аналогичные опасные команды. Каждая команда проверяет параметры, полномочия и допустимость операции. Для потенциально разрушительных действий предусмотрен явный отдельный action/confirmation flow.

## 5. Локальное хранение

### 5.1. SQLite

- Включить `PRAGMA foreign_keys=ON`.
- Использовать WAL, если подтверждено тестами для выбранной версии/настроек.
- Транзакции для всех многотабличных изменений.
- Миграции с порядковыми версиями и тестом upgrade с предыдущей схемы.
- Не изменять старые миграции после публикации; добавлять новые.
- Индексы на FK, sort/filter fields, provider IDs, timestamps.
- FTS5 для title, descriptions, documents, comments (с настраиваемым охватом).
- Хранить даты в UTC; отображать в локальном часовом поясе системы.
- Сгенерированные UUID/ULID локальные IDs; provider IDs сохранять отдельно как opaque strings.
- Учитывать SQLite ограничение на конкурентную запись: один DB writer/serializable transaction strategy, чтения могут быть отдельными соединениями.

### 5.2. Backup/restore

- Кнопка backup вручную.
- Автоматические локальные snapshots по расписанию, которое пользователь может отключить.
- Backup должен быть консистентным с WAL: использовать SQLite backup API или `VACUUM INTO`; запрещено полагаться на обычное копирование только `.db` при работающем WAL.
- В backup включать БД и каталог attachments; manifest с версией схемы, временем, checksum.
- Restore сначала проверяет целостность и версию, затем создаёт резервную копию текущего состояния.
- Не включать credential tokens в backup по умолчанию.
- Экспорт выбранных сущностей в JSON/CSV/Markdown.

### 5.3. Attachments

- Малые метаданные в SQLite; бинарные файлы — в управляемом каталоге данных приложения.
- Проверять размер, MIME по содержимому и расширение; безопасно генерировать имена.
- Не использовать пользовательский путь как непосредственный destination.
- Поддержать ссылку на локальный файл, если пользователь сам выбрал такую опцию.

## 6. Предлагаемая доменная модель

Названия таблиц могут быть изменены, но семантика должна сохраниться.

### 6.1. Core tables

- `workspace`: локальное рабочее пространство (single-user; одна запись допустима в v1).
- `project`: проекты.
- `initiative`: инициативы.
- `milestone`: этапы проекта.
- `issue`: задачи.
- `issue_relation`: связи между задачами.
- `label`, `issue_label`: метки.
- `workflow`, `workflow_status`, `workflow_transition`: workflow.
- `cycle`, `cycle_issue`: циклы.
- `comment`: локальные комментарии и кэшированные внешние комментарии с указанием origin.
- `document`, `document_revision`, `document_link`: документы и их связи.
- `attachment`: вложения.
- `saved_view`: пользовательские представления и сериализованные параметры.
- `activity_event`: история локальных изменений и синхронизированных внешних событий.
- `inbox_item`: уведомления/события пользователя.
- `triage_item`: входящие задачи/внешние Issues.
- `dashboard`, `dashboard_widget`: панели и виджеты.
- `automation_rule`, `automation_run`: правила и журнал исполнения.

### 6.2. Git/provider tables

- `provider_account`: тип провайдера, host/base URL, external account identity, auth state (без токена).
- `repository`: логическая provider-neutral сущность (stable provider ID, host,
  owner/name, metadata, canonical remote URL); remote-only repository не требует path.
- `repository_checkout`: локальные clone/checkout пути, привязанные к repository;
  path уникален в workspace, но не является идентичностью repository.
- `repository_project`: единственный источник истины many-to-many связи
  project/repository, с проверкой принадлежности одному workspace.
- `remote_entity`: тип провайдера, opaque remote ID, repository/account, number, URL, timestamps, minimal metadata and sync state.
- `remote_link`: связь local issue/project/document с remote entity.
- `pull_request_cache`: нормализованные PR fields, branch/head/base, draft/state/mergedAt/review state.
- `commit_cache`: SHA, message, author, date, associated branches/PRs.
- `check_run_cache`: check name/conclusion/status/url/updatedAt.
- `sync_cursor`: cursor/watermark per provider/account/repository/resource/connection.
- `sync_job`: persistent sync/outbox work, priority, attempts, next_attempt_at, state.
- `api_budget_observation`: observed headers and rate-limit snapshots per bucket.
- `api_request_log`: sanitized operation metadata/cost/status/time; never tokens or full private payload by default.
- `http_cache`: ETag/Last-Modified/key/freshness metadata where applicable.

### 6.3. Issue source/ownership

Для задачи явно хранить `origin_kind` (`local`, `github`, `gitlab`, `gitea`) и `source_of_truth_policy`. Для Linked Issue в настройках связи задаётся per-field ownership:
- remote-owned;
- local-owned;
- bidirectional with conflict resolution.

Не синхронизировать каждое поле по умолчанию. В v1 разумный baseline: удалённые title/body/state/labels/comments принадлежат provider, локальные project/cycle/estimate/relations/doc links принадлежат ForgeDesk; пользователь может изменить mapping, если это безопасно реализуемо. Не перетирать удалённые изменения без conflict record.

### 6.4. Идентификаторы

- Local ID — UUID/ULID.
- Human key — `FD-<number>`, локальный и стабильный.
- Remote IDs — opaque strings, не парсить и не считать числовыми.
- Номер Issue/PR — отображаемый natural number в пределах репозитория, не использовать как глобальный PK.
- URL — сохранять как выданный провайдером; не строить вручную по шаблонам.

## 7. Модули и требования

### 7.1. Application shell — P0

Обязательная видимая структура, размеры окна, дизайн-токены, sidebar order, page templates и состояния определены в `06_UI_UX_SPEC.md`. Не заменять их произвольным шаблоном или default theme UI kit.

- Установка и запуск на Windows/Linux.
- Окно, navigation, theme persistence.
- App data directory через платформенные API Tauri.
- Локальная БД создаётся при первом старте.
- Ошибки старта БД/миграций отображаются понятно и не приводят к молчаливому сбросу.
- Приложение не требует сети для старта.

**Приёмка:** отключённая сеть не препятствует запуску, локальному чтению/созданию тестовых задач и закрытию программы. Shell проходит UI Gate 0 из `06_UI_UX_SPEC.md`.

### 7.2. Issues/Workflow — P1

Реализовать все поля и операции из `01_PRODUCT_PROJECT.md`. Подзадачи и relations проверяются на циклы. Статус имеет категорию. Переходы логируются. Batch edit — атомарная транзакция.

**Приёмка:** создать/редактировать/архивировать задачи; массово изменить priority/labels/status; связать задачи; создать вложенную подзадачу; фильтровать по любым основным полям; перезапустить приложение и получить те же данные.

### 7.3. Views/Search — P1

- List и Board в первой рабочей итерации.
- Table/Calendar/Timeline — расширение P2.
- Saved Views с версионируемой схемой JSON и валидацией.
- Фильтры AND/OR и группы.
- SQLite FTS5 для поиска.
- Виртуализация больших списков.

**Приёмка:** сложный фильтр `(priority = urgent OR priority = high) AND status != done AND project = X` возвращает ожидаемые строки; view восстанавливается после перезапуска.

### 7.4. Projects/Initiatives/Cycles/Documents — P1/P2

Реализовать project/initiative hierarchy, milestones, cycles, templates, documents, revision history и ссылки. Progress aggregation имеет чётко документированную формулу и не считает дублирующиеся связи дважды.

**Приёмка:** создать инициативу с несколькими проектами, проект с milestones, cycle с задачами; изменить состояние задач и проверить корректное обновление прогресса; экспортировать/importировать Markdown документ.

### 7.5. Local Git — P1

Использовать `std::process::Command` или специализированный безопасный процессный wrapper. Не использовать `sh -c`, PowerShell command concatenation или строковую интерполяцию для пользовательских значений.

Команды v1: detect git, version, status, current branch, branch list, create branch, checkout/switch, log summary, diff summary, open terminal/folder. Перед branch switch/create проверять dirty working tree. При dirty state — показать предупреждение и не терять изменения.

**Приёмка:** тесты с временным git repo проверяют create/switch branch, dirty guard, unusual path with spaces/unicode, отсутствие shell injection.

### 7.6. Git provider abstraction — P0/P1

Определить provider contract (Rust traits или эквивалент):
- authenticate/validate connection;
- account/profile discovery;
- list repositories/get repository;
- list/get/create/update issues;
- list/get PR/MR;
- list/get comments/reviews;
- list commits/branches/tags/releases;
- checks/pipelines;
- provider-specific project features;
- capability discovery;
- rate-limit observation;
- pagination abstraction.

Не требовать реализации всех методов каждым провайдером. Методы с неподдерживаемой функцией возвращают explicit `UnsupportedCapability`, а не пустой успешный результат.

### 7.7. GitHub integration — P1/P2

Полный объём и обязательные проверки находятся в `03_GITHUB_API_BUDGET_AND_SYNC.md`. GitHub integration начинается на раннем этапе с подключения и чтения списка репозиториев. Основной пользовательский функционал (Issues, PR, commits, checks, reviews, Projects V2) реализуется порциями.

### 7.8. Sync Engine — P0/P1

- persistent outbox/queue;
- отдельный state machine для sync jobs;
- дедупликация;
- cursors/watermarks;
- prioritization;
- retry/backoff;
- pause/resume;
- conflict records;
- graceful shutdown;
- status exposed to UI;
- no networking at app startup if integration disabled.

### 7.9. Inbox/Triage/Automations — P2

Все внешние события сначала нормализуются и дедуплицируются. Automation execution должен быть идемпотентным, с protection against loops and duplicate runs. Любое удалённое write action требует нужной permission и user policy.

### 7.10. Insights/Dashboards — P2

Все графики строятся из локальных агрегатов. Если часть данных ещё не синхронизирована, UI показывает `data through <timestamp>` и неполноту. Нельзя выдавать кэшированные сведения за live data.

### 7.11. AI Extension — P3, optional

- Собственный Cargo feature `ai-extension` (или отделяемый crate).
- Выключен по умолчанию в первой core сборке.
- UI и команды регистрируются только если включён feature + runtime setting enabled.
- Core не импортирует AI crates.
- MCP через STDIO — только при включённом AI extension и явном старте/настройке.
- В отключённом состоянии не загружать API keys и не запускать polling/workers.
- AI-вызовы показывают, какой контекст будет отправлен.
- AI mutation suggestions требуют подтверждения.

**Приёмка:** собрать приложение без AI feature; проверить запуск, полный Core workflow и отсутствие AI-команд/пунктов меню/сетевых AI-вызовов. Второй тест — сборка с feature, но runtime toggle off: модуль не активен.

### 7.12. UI/UX implementation and visual acceptance — P0/P1

`06_UI_UX_SPEC.md` — нормативная спецификация frontend. Она обязательна для всех этапов, затрагивающих UI. DeepSeek обязан реализовать её буквально в видимой части приложения: shell geometry, sidebar order, color tokens, typography, density, components, screen layouts, interactions, loading/empty/error/offline states and keyboard shortcuts.

Запрещено самостоятельно менять визуальный язык, навигацию, стандартные размеры, компоновку sidebar/content/detail pane, default view layouts или заменять их шаблоном UI framework. Внутреннюю реализацию компонентов можно выбирать самостоятельно, если итоговое поведение и визуальный результат совпадают со спецификацией.

UI выполняется последовательно через UI Gates 0–5, описанные в `06_UI_UX_SPEC.md`. Stage 1 обязан завершиться shell + design system, а не только пустым окном. Каждый Gate требует screenshot evidence на заданных viewport/theme states и отчёт PASS/PARTIAL/FAIL. Если среда не позволяет сделать скриншоты, visual check честно помечается SKIPPED; нельзя объявлять его пройденным.

**Приёмка:** окно соответствует фиксированным измерениям и токенам; Dark и Light темы работают; sidebar и detail pane ведут себя как описано; локали en/ru поддержаны; клавиатурная навигация работает; screen states реализованы; нет inert controls/mock data, выдаваемых за реальные функции; screenshot visual gate приложен. Подробный checklist — разделы 12–15 `06_UI_UX_SPEC.md`.

## 8. GitHub Authentication and Permissions

- Целевой механизм — публичный GitHub App user authorization flow для desktop-клиента.
- Для constrained desktop client допускается Device Flow, если он включён в GitHub App registration; не опрашивать token endpoint чаще `interval`, уважать `slow_down` (добавляет 5 секунд) и expiration.
- На этапе реализации проверить возможность Authorization Code + PKCE для выбранной GitHub App конфигурации и предпочесть его, если он штатно поддерживается без доставки client secret. Не включать device flow без причины.
- Никогда не включать GitHub App private key или client secret в desktop distribution, repo, logs или config.
- Client ID не является секретом, но token/refresh token — секреты.
- Хранить токены в OS credential store (Windows Credential Manager, macOS Keychain, Linux Secret Service/libsecret). Если недоступно — безопасно отказать в persistent storage и предложить повторную авторизацию; не записывать токен plaintext в SQLite.
- Учитывать, что user access token ограничен пересечением прав пользователя и GitHub App; получение токена не означает доступ ко всем репозиториям.
- Просить только минимальные permissions, необходимые включённым возможностям. Разделить read и write permissions, где платформа позволяет.
- На revoke/401 прекратить sync, очистить активное соединение и предложить re-auth. Не повторять бесконечно.

## 9. Ошибки, журналы, диагностика

- Стабильные error categories: validation, local_db, git, auth, permission, network, provider, rate_limit_primary, rate_limit_secondary, conflict, unsupported_capability, internal.
- В UI сообщение для пользователя + technical detail для диагностики.
- Logs структурированные, с уровнями; не включают tokens, auth headers, raw private issue bodies, repository secrets.
- API Monitor показывает последние operation labels, cost/count/latency/status и rate-limit headers (без тел запросов).
- Пользователь может экспортировать sanitised diagnostics.

## 10. Безопасность

- Минимальные Tauri capabilities.
- Не разрешать frontend произвольный filesystem/network/shell access.
- Разрешения на файловые операции ограничивать выбранными путями.
- Shell integration — только allow-listed открытие приложений/терминала либо отдельный подтверждённый путь.
- Параметризованные SQL statements.
- Проверка путей и защиты от path traversal.
- Credentials только в системном secure store.
- Автоматическое удаление локального token cache при disconnect/revoke.
- Никакой обязательной телеметрии.
- Экспорт логов/данных не включает секреты по умолчанию.

## 11. Тестирование и качество

### 11.1. Обязательные слои тестов

- Unit: domain rules, filters, relations, state machine, API budget maths, retry logic, cache keys.
- DB integration: migrations, transactions, foreign keys, FTS, backup/restore.
- Provider contract tests: общий контракт на mocked HTTP responses.
- GitHub API tests: fixtures с нормальными и ошибочными GraphQL/REST responses, pagination, headers, GraphQL HTTP 200 with errors.
- Sync integration: restart-safe outbox, cursor resume, duplicate event, ambiguous mutation.
- UI tests: main flows and offline startup.
- Manual smoke: Windows и Linux builds.

### 11.2. Нельзя

- Завершать этап при неуспешном `cargo test`/frontend tests/build.
- Подменять реальные GitHub операции фиктивным локальным массивом и объявлять интеграцию готовой.
- Удалять/ослаблять тест без причины и описания.
- Игнорировать flaky test — сначала классифицировать и исправить.
- Скрывать warnings, влияющие на correctness/security.

### 11.3. Минимальный quality gate каждого этапа

1. `cargo fmt --check` (или `cargo fmt` перед коммитом).
2. `cargo clippy --all-targets --all-features -- -D warnings` там, где совместимо; для optional features отдельные build lanes.
3. `cargo test`.
4. Frontend lint/typecheck/test/build.
5. Integration tests для затронутого модуля.
6. Краткий ручной smoke test.
7. Обновлённая документация и changelog.

Если all-features включает необязательные внешние зависимости, добавить отдельную проверку Core без AI и расширенную проверку с AI.

## 12. Этапы реализации и Definition of Done

Реализация не должна вестись как один огромный запрос. Каждый этап заканчивается рабочим состоянием.

### Stage 0 — Repository reconnaissance & architecture lock

- Инвентаризировать репозиторий (если он уже создан), версии Rust/Node/Tauri, существующие файлы, CI и conventions.
- Если repo пустой — создать минимальный skeleton.
- Сформировать `docs/architecture/overview.md`, `docs/decisions/ADR-0001.md` и план этапов.
- Зафиксировать точные версии инструментов после проверки актуальной документации.
- Никаких функций продукта на этом этапе, кроме запуска skeleton.

DoD: проект собирается на целевых ОС хотя бы в CI; есть схема архитектуры и журнал решений; нет лишнего backend.

### Stage 1 — Desktop/Core/SQLite

- Tauri shell, Vue application shell, themes, navigation placeholders.
- SQLite path, migrations, repositories, backup/restore.
- Settings and error surfaces.

DoD: офлайн запуск, БД создаётся, migrations проходят, backup/restore проверены.

### Stage 2 — Issues/Workflow/Search/Views

- Issue CRUD, statuses, priority, labels, relations, sub-issues, history.
- List/Board, filters, saved views, global search.

DoD: базовый локальный workflow полностью работает без GitHub и AI.

### Stage 3 — Projects/Initiatives/Cycles/Documents

- Project, initiative, milestone, cycle, templates, docs, progress.

DoD: можно планировать несколько связанных проектов и экспортировать документы.

### Stage 4 — Local Git + GitHub connection + API Budget foundation

- Local Git operations (safe subset).
- GitHub app auth/credentials.
- API Budget Manager, serialized queue, request log, API Monitor.
- Repository discovery and selection.

DoD: подключение, выбор repo, просмотр списка, корректные budgets, offline behaviour, tests on rate-limit fixtures.

### Stage 5 — GitHub Issue synchronization

- GitHub Issues/labels/comments.
- Local↔remote linking, field ownership, conflict UI, cursor sync.

DoD: импорт, создание/изменение с user intent, no duplicate on ambiguous mutation, incremental sync.

### Stage 6 — Pull Requests/Commits/Checks/Reviews

- PR list/details, commits, changed files on demand, reviews, checks, status automation hooks.

DoD: связать PR с задачей, увидеть statuses and review, safely perform supported actions.

### Stage 7 — Inbox/Triage/Automation/Analytics

- Event normalization, inbox, triage, rules, insights, dashboards, roadmap.

DoD: end-to-end issue → branch → PR → merge → task completion works with observable automation history.

### Stage 8 — GitLab/Gitea providers

- Capability-driven adapters and contract tests.

DoD: core operations work with fixtures and at least one live test environment per provider where available; limitations are visible.

### Stage 9 — Optional AI Extension

- AI providers, context selection, MCP, approvals, build optionality.

DoD: Core build without AI; runtime off yields no AI activity; extension build tests; no AI dependency in default app.

### Stage 10 — Stabilization/Release

- Performance, accessibility, migration test, signed installers if signing credentials available, crash handling, export/import, user docs.

DoD: release candidate tested on Windows/Linux, full regression suite green, known limitations documented.

## 13. Git workflow and delivery

- Предлагать небольшие логически завершённые changesets.
- Не переписывать не затронутые модули.
- Не делать массовый refactor одновременно с feature, если это не нужно для неё.
- PR/commit messages содержат ссылку на issue/этап, если он есть.
- Не добавлять GitHub Actions self-hosted runner как предположение. CI можно настроить на GitHub-hosted runners, если репозиторий использует GitHub Actions.
- В конце каждой итерации выдавать summary: изменённые файлы, реализованное поведение, команды проверки и результат, оставшиеся риски.

## 14. Правило работы без бесконечных уточнений

DeepSeek должен принимать решения по умолчанию в рамках этого ТЗ. Не задавать вопрос по каждой мелочи, цвету, имени метода, расположению файла, библиотеке или внутренней реализации.

Если параметр не задан:
1. выбрать безопасный и простой вариант, соответствующий архитектуре;
2. записать выбор в `docs/decisions/` или `docs/architecture/assumptions.md`;
3. продолжить разработку;
4. вынести в вопрос только то, что действительно блокирует работу или создаёт необратимое внешнее действие/риск потери данных.

Запросить согласование только для:
- необратимых/потенциально разрушительных операций;
- реального изменения GitHub permissions или удаления remote data;
- выбора, который существенно меняет product scope, storage format или sync ownership;
- затратных внешних действий, которые нельзя безопасно выполнить без пользователя.

## 15. Итоговые критерии готовности продукта

ForgeDesk нельзя объявить готовым только на основании наличия главного окна и списка задач. Для полноценного релиза должны выполняться критерии:

- локальная работа полностью возможна без сети;
- данные переживают перезапуск и backup/restore;
- issues/projects/cycles/docs/views/search работают;
- GitHub integration использует real API, budgets, pagination, ETags and recovery logic;
- API requests are serialized/adaptive and failures are visible;
- linked objects preserve ownership and conflict history;
- PR/commits/checks/reviews связаны с local issues;
- GitLab/Gitea support uses provider capabilities;
- AI выключен по умолчанию и может быть исключён из build;
- тесты, сборки и документация соответствуют текущему этапу;
- нет скрытой обязательной инфраструктуры или сетевой зависимости.
