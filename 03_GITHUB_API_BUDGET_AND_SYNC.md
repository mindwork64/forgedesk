# 03. GitHub API Budget Manager, GraphQL/REST и Sync Engine

**Приоритет:** P0/P1 — критическая часть архитектуры  
**Дата сверки официальных правил:** 15 ноября 2026 года  
**Важно:** числа ниже являются ориентирами документации GitHub на дату сверки. Они не должны становиться жёсткими константами, заменяющими анализ реальных ответов API.

## 1. Цель

Спроектировать и реализовать безопасную интеграцию GitHub, которая:
- не расходует API-квоты бесконтрольно;
- не делает тяжёлые запросы без пользовательской необходимости;
- использует локальный кэш и инкрементальную синхронизацию;
- правильно различает GraphQL primary points, REST primary requests и общие secondary limits;
- умеет адаптировать скорость синхронизации по фактическим ответам API;
- не запускает бесконечные циклы повторов;
- не создаёт дубликаты после сетевых таймаутов на мутациях;
- даёт пользователю прозрачную картину использования API.

Подсистема обязательна независимо от того, включён ли AI Extension.

## 2. Базовая модель ограничений GitHub

### 2.1. Primary limits — отдельные бюджеты

Для стандартной пользовательской авторизации на GitHub.com документация указывает общие ориентиры:

- REST API: обычно 5 000 запросов в час на пользователя.
- GraphQL API: обычно 5 000 points в час на пользователя.
- Для некоторых GitHub Enterprise Cloud app-owned сценариев могут действовать повышенные лимиты.
- GitHub App installation access token имеет собственную модель лимитов, зависящую от installation/repository/user и типа организации.
- GitHub App user access token использует лимит пользователя, а не отдельную квоту для ForgeDesk.

**Не объединять REST primary и GraphQL primary в один числовой счётчик:** это разные основные buckets. Но и не считать их квоты собственностью ForgeDesk: другие приложения и PAT/OAuth/GitHub Apps, работающие от имени этого же пользователя, могут расходовать соответствующие пользовательские buckets. Особенно важно не предполагать, что локальный счётчик отражает активность других приложений.

### 2.2. Secondary limits — общие ограничения нагрузки

Документация GitHub описывает, среди прочего, следующие ограничения/ориентиры:

- не более 100 одновременно выполняющихся запросов, общий предел REST + GraphQL;
- до 900 REST secondary points в минуту на endpoint;
- до 2 000 GraphQL secondary points в минуту на GraphQL endpoint;
- ориентир CPU time — не более 90 секунд на 60 секунд реального времени, в том числе не более 60 секунд GraphQL CPU time;
- общий ориентир content-generating requests — не более 80 в минуту и 500 в час; у отдельных endpoints ограничения могут быть ниже;
- некоторые endpoints (например Search) имеют собственные более строгие лимиты;
- secondary limits могут быть неизвестными заранее и меняться без уведомления.

Для подсчёта secondary points документация указывает как типовые значения: GraphQL query без mutation — 1 point, GraphQL mutation — 5 points; большинство REST GET/HEAD/OPTIONS — 1 point, большинство POST/PATCH/PUT/DELETE — 5 points. Отдельные REST endpoints могут иметь другую/непубличную стоимость.

### 2.3. Внутренняя политика ForgeDesk строже лимита GitHub

ForgeDesk предназначен для одного пользователя и не нуждается в высокой параллельности. Поэтому:

- по умолчанию максимум **один in-flight GitHub API request** на credential principal;
- GraphQL и REST используют общий dispatcher и общую secondary guard;
- все mutative GitHub requests дополнительно разделяются минимум одной секундой;
- никакие фоновые процессы не создают конкурирующие независимые очереди;
- допустимое изменение concurrency возможно только после измерений и доказательства необходимости; не повышать его ради ускорения начальной синхронизации.

Лимит 100 из документации GitHub — внешний верхний предел, а не рекомендуемая параллельность ForgeDesk.

## 3. Что считать источником истины о лимитах

### 3.1. REST response headers

Сохранять и анализировать:
- `x-ratelimit-limit`;
- `x-ratelimit-remaining`;
- `x-ratelimit-used`;
- `x-ratelimit-reset` (UTC epoch seconds);
- `x-ratelimit-resource`;
- `retry-after`, если присутствует;
- HTTP status;
- `date`;
- для conditional requests — `etag`, `last-modified`.

### 3.2. GraphQL response headers и `rateLimit`

GraphQL primary headers используют те же `x-ratelimit-*` имена, но `remaining/limit/used` обозначают **points**, а не число HTTP-запросов. Для query operations следует включать объект `rateLimit` с `cost`, `remaining`, `limit`, `used`, `resetAt`, когда это практически возможно. `rateLimit.cost` позволяет измерить реальную стоимость конкретной операции.

Не выполнять отдельный запрос только ради проверки лимита перед каждым запросом. Предпочитать headers реального запроса и встроенный `rateLimit` в полезной query. Для mutation использовать headers и последующий обычный query, если требуется дополнительное наблюдение.

### 3.3. `/rate_limit`

REST `GET /rate_limit` допустим для начальной диагностики или явного ручного запроса пользователя. Не вызывать его постоянно: ответные headers являются более актуальными для конкретного запроса, а сам endpoint, хотя и не расходует primary quota, всё ещё может учитываться для secondary limits.

### 3.4. Наблюдаемость вместо ложной точности

GitHub предупреждает, что значения могут различаться между регионами/обработчиками запросов. Поэтому:
- последний фактический response header — основа текущей оценки;
- локальные счётчики показывают расход, который ForgeDesk наблюдал сам;
- внешний расход другими приложениями оценивается только по изменению observed remaining;
- UI не должен обещать точный свободный остаток между запросами;
- каждый snapshot содержит `observed_at`, bucket, endpoint/resource и источник данных.

## 4. Data model API Budget Manager

Для каждой credential identity хранить:

### `api_budget_observation`
- `id`;
- `provider_account_id`;
- `api_family`: `github_rest`, `github_graphql`, `github_search`, `github_code_search`, `unknown` (дополняется по фактическому `x-ratelimit-resource`);
- `resource_name`;
- `limit_value`;
- `remaining_value`;
- `used_value`;
- `reset_at_utc`;
- `observed_at_utc`;
- `http_status`;
- `source`: response_headers / graphql_rate_limit / manual_probe;
- `is_estimate`;
- `secondary_pause_until`;
- `notes` (без секретов).

### `api_request_log`
- operation name / internal operation code;
- provider account;
- API family/resource;
- request class (`interactive_read`, `interactive_write`, `foreground_refresh`, `background_sync`, `manual_full_sync`, `reconciliation`);
- request fingerprint (без токенов и private payload);
- GraphQL estimated and actual cost;
- HTTP status;
- GraphQL error category;
- latency;
- response received time;
- retry attempt;
- rate headers snapshot;
- payload size (если безопасно измерять);
- result state.

Не сохранять Authorization headers, raw tokens, весь private issue body, diff или комментарии в API log.

## 5. Dispatcher и scheduler

### 5.1. Единая очередь на GitHub principal

GraphQL и REST должны проходить через один `GitHubRequestDispatcher`. Не допускается независимый HTTP client из модулей Issues, PR, Projects, Checks и Settings.

Каждый request содержит:
- API family/resource;
- operation type;
- priority;
- estimated cost/size;
- retry policy;
- read/write classification;
- user initiated / background flag;
- idempotency/reconciliation strategy;
- cancel token;
- deadline (если это интерактивное действие).

### 5.2. Приоритеты

1. `P0_INTERACTIVE_WRITE`: действие, прямо запрошенное пользователем (создать/изменить Issue, comment, review).
2. `P1_INTERACTIVE_READ`: открыть экран или вручную обновить конкретную сущность.
3. `P2_ACTIVE_CONTEXT`: данные открытого проекта/задачи/PR и активного локального репозитория.
4. `P3_BACKGROUND_SYNC`: обновление отслеживаемых сущностей.
5. `P4_MANUAL_BULK_SYNC`: явно запущенная пользователем полная/историческая синхронизация.

P4 не должен вытеснять уже выполняемое пользовательское действие. Пользователь должен видеть и иметь возможность отменить bulk sync.

### 5.3. In-flight и pacing

- `max_in_flight_per_principal = 1` по умолчанию.
- REST и GraphQL делят этот слот.
- Между mutative requests выдерживать минимум 1 секунду; применять также при переходе между REST и GraphQL, потому что content generation учитывает оба API.
- Для read operations не вводить безусловную секундную задержку, если нет сигналов secondary limit; регулировать адаптивно по latency, budget и ошибкам.
- Отмена запроса не должна приводить к потере его persisted job state.
- При выходе из приложения остановить новые запросы, дождаться безопасного завершения текущего или пометить его outcome как unknown, сохранить очередь и закрыть процесс.

### 5.4. Background quota policy

Начальные настройки (пользователь может изменить):
- автоматическая фоновая синхронизация расходует не более 10% от наблюдаемого primary limit соответствующего bucket за reset window;
- дополнительно всегда сохраняется защитный остаток: начально 20% от последнего наблюдаемого `limit`, если это не мешает пользовательским действиям;
- реальный background allowance рассчитывается отдельно для каждого primary bucket по формуле: `min(0.10 * observed_limit, max(0, observed_remaining - 0.20 * observed_limit))`;
- если `remaining` неизвестен/устарел, фоновый allowance считается нулевым до следующего достоверного observation;
- interactive actions могут использовать бюджет сверх background allowance, но не могут обходить реальные provider rate limits;
- при низком/неопределённом remaining фоновая очередь ставится на паузу;
- пользователь может вручную запустить полный sync и видеть прогноз по последней измеренной стоимости, но прогноз явно маркируется как оценка.

Формула должна быть реализована и покрыта unit tests. Не делать ошибочное предположение, что `limit - remaining` равняется расходу только ForgeDesk.

### 5.5. Adaptive scheduler

Учитывать:
- reset window;
- последнюю observed remaining;
- стоимость предыдущих аналогичных запросов;
- p50/p95 latency;
- время последней синхронизации сущности;
- пользовательскую активность;
- размер backlog;
- recent `403/429` и GraphQL errors;
- secondary pause.

Если запросы стабильно дешёвые и API здоров — использовать заданные интервалы. Если remaining быстро уменьшается или появляются rate-limit signals — уменьшать частоту, откладывать фоновые задачи и не запускать bulk. Scheduler должен иметь минимальный интервал и jitter, чтобы не создавать частые повторяющиеся bursts.

## 6. GraphQL design requirements

### 6.1. General rules

- GraphQL — основной путь чтения/изменения данных, когда нужная операция поддерживается.
- Каждое connection поле обязано указывать `first` или `last`.
- `first/last` не более 100 по требованиям GitHub.
- По умолчанию summary pages 25–50 records; 100 использовать только для явно ограниченной пользовательской страницы, если cost estimate приемлем.
- Не превышать общий лимит 500 000 nodes в одном запросе. Внутренняя политика ForgeDesk должна быть намного строже: целевая верхняя граница 10 000 predicted nodes на один query; операция, которая превышает внутреннюю границу, должна быть разбита на несколько страниц/запросов.
- Не делать глубокие вложенные connections: repositories → PRs → reviews → comments → files в одном запросе запрещено.
- Сущности загружаются слоями: summary list → detail → comments/reviews/files/diff при открытии.
- Все пагинационные connections используют курсоры и сохраняют `endCursor`, `hasNextPage`.
- Стабильный sort order и persisted cursor.
- Не получать весь `repository` object для каждой задачи; переиспользовать уже сохранённую связь по ID.
- GraphQL queries должны выбирать минимальный набор полей, необходимый текущему экрану.
- REST requests должны явно передавать поддерживаемый `X-GitHub-Api-Version`; версию REST API выбирать централизованно по актуальной stable version на момент реализации/release (в текущей документации приведён пример `2026-03-10`, но это значение не должно дублироваться в коде и требует повторной проверки).

### 6.2. Query registry

Каждая операция должна иметь зарегистрированное описание:
- стабильный operation name;
- query/mutation type;
- цель и вызывающий UI flow;
- выбранные поля;
- connection page sizes;
- predicted node count;
- conservative estimated primary cost;
- expected response type;
- cache lifetime/freshness class;
- required GitHub permissions;
- retry classification;
- tests/fixtures.

Не формировать произвольные огромные GraphQL строки из UI. Использовать query files/constants и variables. Не включать в query персональные данные/поля, не используемые приложением.

### 6.3. Query cost

- Для типовых операций рассчитывать приблизительную стоимость до отправки на основании connections.
- После query сохранять фактический `rateLimit.cost` и обновлять статистику для этого operation name.
- Сравнивать prediction и actual cost; если расхождение систематическое — корректировать estimator.
- При ошибке `RATE_LIMITED`, `RESOURCE_LIMITS_EXCEEDED`, timeout или oversized response — не выполнять тот же запрос неизменённым автоматически; уменьшить page size/плоскость выборки.
- Минимальная cost GraphQL call — 1 point; не пытаться искусственно объединять несколько дорогих независимых сущностей ради уменьшения HTTP request count, если это увеличивает complexity и nodes.

### 6.4. GraphQL errors

HTTP 200 не означает успех операции. Всегда анализировать `errors` и `data`:
- rate-limit error;
- permission/forbidden;
- not found/null due to access;
- validation/schema error;
- timeout/resource limit;
- partial data.

Частичные данные можно применять только если каждая применяемая часть валидна и не нарушает атомарность доменного изменения. Для read query допускается сохранить валидные части и показать incomplete state; для mutations нельзя трактовать partial response как success без reconciliation.

### 6.5. Pagination and incremental update

- Сохранять cursors per repository + connection type + filters/sort definition.
- Не начинать full pagination с начала при каждом запуске.
- Использовать сортировку по `UPDATED_AT` там, где connection schema поддерживает её.
- Для обновлений использовать watermark с overlap window, чтобы не потерять изменения на границе времени.
- Upsert by provider opaque ID.
- Cursor advancement выполняется только после успешного применения всей страницы транзакционно.
- При удалённом объекте сохранять tombstone/sync state, не удалять локальную задачу автоматически.
- При смене query shape/filter не переиспользовать старый cursor.

## 7. REST API requirements

### 7.1. REST как вспомогательный API

REST использовать, когда:
- GraphQL не предоставляет нужную операцию/поле;
- REST endpoint позволяет существенно эффективнее выполнить задачу;
- нужен conditional GET/ETag;
- provider feature/documentation прямо рекомендует REST.

Не дублировать чтение одной и той же сущности обоими API без причины. Для каждой операции явно указать выбранный transport и обоснование.

### 7.2. Conditional requests / ETag

Для поддерживающих GET endpoints:
1. Выполнить authenticated GET.
2. Сохранить `ETag` и при наличии `Last-Modified` вместе с точным canonical URL/query params, API version, authorization principal hash и relevant vary metadata.
3. При повторной проверке отправлять `If-None-Match` и/или `If-Modified-Since`.
4. `304 Not Modified` не должен очищать/перезаписывать существующие данные.
5. Использовать стабильные endpoint URLs, filters, page size и sort order, чтобы кэш и ETag оставались валидными.
6. Кэшировать только для того же пользователя/прав доступа; не переиспользовать ответ между разными credential identities.

GitHub указывает, что корректно авторизованный conditional request с ответом `304` не расходует primary REST rate limit. Он всё ещё должен проходить общий dispatcher/secondary guard.

Не применять conditional request к mutation, если конкретный endpoint прямо не документирует такую возможность.

### 7.3. Pagination

REST pagination выполнять по `Link` headers, не конструировать next URL вручную. Сохранять next URL/cursor как opaque value и соблюдать redirect behavior.

### 7.4. Search endpoints

Не использовать Search API для полной синхронизации всех Issues/PR. У Search endpoints отдельные лимиты и более строгие ограничения. Для известного repository scope предпочитать repository connections/lists с фильтрами и cursor pagination. Search — только для настоящего пользовательского поиска или сценария, для которого это наиболее подходящий API.

## 8. Sync strategy без собственного сервера и Webhooks

### 8.1. Режим работы

Базовая версия — polling, но адаптивный и минимальный. Никакой публичный сервер/relay не обязателен. Webhook relay может быть дополнительным расширением в будущем, но не входит в Core и не требуется для корректности данных.

### 8.2. Sync tiers

Начальные значения — defaults, адаптируемые scheduler:
- активные PR/Checks, относящиеся к текущим задачам или открытым экранам: ориентир 10–15 минут, только если background budget позволяет;
- активные Issues и метаданные выбранных repositories: 30–60 минут или по факту открытия/ручного refresh;
- неактивные репозитории: несколько часов/по факту обращения;
- закрытые/старые Issues, вся история comments/reviews/commits: по запросу или по необходимости;
- подробный PR diff/files/review comments: lazy load при открытии соответствующей вкладки;
- полная историческая синхронизация: только вручную.

Scheduler должен увеличивать интервалы при низкой квоте/ошибках и не продолжать polling при закрытом приложении.

### 8.3. Lifecycle triggers

- App start: показать локальное состояние сразу; не запускать массовую синхронизацию.
- Connection established: validate token/profile, fetch essential rate/resource observations, затем минимальный discovery.
- App foreground: обновить открытый контекст, если stale и budget разрешает.
- Open entity: обновить только эту сущность/видимые детали.
- Manual refresh: приоритетная задача на соответствующую сущность/экран.
- Close: graceful pause and persisted queue.
- Network restored: resume delayed jobs through scheduler.

### 8.4. Incremental synchronization

- Получать новые/изменённые сущности по stable sort/update window/cursor, а не переобходить всю историю.
- Использовать overlap window при time watermark.
- Сохранять per-repository per-resource cursor.
- Сохранять `last_successful_sync_at`, `last_attempt_at`, `next_due_at`, page cursor, watermark.
- Обновлять local cache через upsert and transaction.
- Не обновлять `last_successful_sync_at` при частичном/ошибочном выполнении.
- Сохранять причину неполноты.

## 9. Mutation safety и duplicate prevention

Это обязательный механизм.

### 9.1. Outbox state machine

Для записи на GitHub:
`pending → dispatching → succeeded | failed_definite | outcome_unknown → reconciling → succeeded | retry_safe | needs_user_action`.

### 9.2. Ambiguous outcome

Если произошёл timeout/connection reset после отправки mutation и до получения надёжного ответа:
- не повторять mutation автоматически;
- отметить `outcome_unknown`;
- выполнить read/reconciliation запрос с низкой частотой/приоритетом;
- проверить по remote ID, URL, expected state, branch/commit marker или доступному fingerprint;
- если remote object найден и совпадает — пометить как success;
- если подтверждено, что объект не создан/изменение не применено — разрешить retry;
- если проверить нельзя — показать пользователю `Needs reconciliation` и не создавать потенциальный дубль.

### 9.3. Operation-specific deduplication

- Создание Issue: хранить local operation UUID и, если допустимо, непротиворечивый correlation marker в локальных метаданных/описании только при согласии пользователя; не добавлять скрытые маркеры в публичные описания без нужды. После таймаута искать по автору/репозиторию/времени/заголовку/содержимому, затем просить пользователя разрешить неоднозначность.
- Comment creation: при неопределённом исходе не повторять автоматически без проверки; комментарии могут быть одинаковыми, поэтому fingerprint включает body, author, parent issue и временное окно, но совпадение не является абсолютным доказательством.
- Review submission: сверить remote review state и commit SHA.
- State/label update: повторяемые idempotent set operations можно повторить после проверки текущего значения; не использовать toggle semantics без read.
- Merge: не повторять без проверки mergedAt/merge commit.

## 10. Error handling / backoff

### 10.1. Primary limit exhausted

Если `x-ratelimit-remaining=0`:
- остановить соответствующий primary bucket до `x-ratelimit-reset`;
- не делать тестовый запрос до reset;
- остальные независимые buckets могут продолжать работу только если secondary guard healthy;
- UI показывает reset time и какое действие заблокировано.

### 10.2. Secondary rate limit

При явном secondary limit или `403/429` с указанием secondary:
- если есть `Retry-After`, соблюдать его;
- если primary remaining = 0 — ждать reset;
- иначе ждать минимум 60 секунд;
- при повторении — exponential backoff с jitter;
- после ограниченного количества повторов (начально 3 automatic attempts) перевести job в delayed/needs user action;
- не отправлять другие запросы через общий dispatcher до истечения общей secondary pause.

### 10.3. GraphQL rate-limit errors

GraphQL может вернуть HTTP 200 с `errors` и сообщением о rate limit. Ошибка должна распознаваться по структурированному содержимому, а не только по HTTP status. Для query можно безопасно повторить позже после паузы; mutation — только через outcome reconciliation.

### 10.4. Network / 5xx

- timeout/network unavailable: bounded retries для безопасных reads; мутации становятся outcome_unknown, если запрос мог быть отправлен.
- 502/503/504: retry reads с exponential backoff и ограничением; mutation только после reconciliation.
- 401: остановить provider queue, отметить auth expired/revoked, предложить reconnect.
- 403: проверить permission vs rate limit; не считать все 403 rate limit.
- 404 для private resource может означать отсутствие доступа, а не отсутствие объекта; не повторять бесконечно.
- 422/validation: не повторять без изменения запроса, показывать детали.

### 10.5. Неизвестные ошибки

Сохранять sanitized diagnostics, останавливать конкретный job, не блокируя локальную работу. Повторять только по классифицированной политике.

## 11. API Monitor UX

Показывать:
- connection/auth state;
- GraphQL primary: observed limit/remaining/used/reset;
- REST primary: observed limit/remaining/used/reset;
- REST special resources (search/code_search/прочие), если GitHub предоставляет их через headers;
- последний observation timestamp;
- observed cost и p50/p95 cost по GraphQL operation;
- requests ForgeDesk per bucket за текущий reset window;
- background vs interactive consumption;
- jobs waiting/paused/failed/outcome_unknown;
- next sync due;
- secondary pause status;
- последнюю причину slowdown/pause;
- кнопку sync current screen/entity;
- ручной full sync с estimated cost/pages и предупреждением;
- кнопку export sanitized diagnostics.

UI обязан различать:
- фактические значения из последних headers;
- локально измеренный расход ForgeDesk;
- оценку/forecast;
- неизвестные данные.

Не показывать точный percentage gauge как гарантию, если данных недостаточно. В tooltip пояснить, что другие приложения аккаунта также могут расходовать пользовательскую квоту.

## 12. Тестовые сценарии API Budget и Sync

Обязательно создать mock fixtures и unit/integration tests для:

1. REST `200` с обычными `x-ratelimit-*`.
2. REST `304` с ETag — body не запрашивается/не перезаписывается; primary request accounting не увеличивает расход ForgeDesk.
3. GraphQL успешный `200` с `rateLimit.cost/remaining/resetAt`.
4. GraphQL `200` с `errors` и rate limit message.
5. GraphQL partial data + errors.
6. REST `403` + `x-ratelimit-remaining=0` — wait reset.
7. REST `429` + `Retry-After` — строго выдержать время.
8. `403/429` secondary без Retry-After — не менее 60 секунд, затем backoff.
9. Повтор secondary error — растущий backoff и остановка после установленного предела.
10. REST и GraphQL одновременно запрошены из разных UI modules — dispatcher запускает только один.
11. Mutations separated by at least 1 second.
12. Rate limit изменён другим внешним приложением: следующее response header заменяет оценку, фоновые jobs сокращаются.
13. `x-ratelimit-reset` parsing UTC epoch and local timezone display.
14. GraphQL `first: 100` valid; `first: 101` rejected by local validator before network.
15. Missing pagination argument rejected by query check/contract tests.
16. Predicted query node budget above 10,000 splits or blocks before network.
17. Persisted cursor survives restart; page applies transactionally.
18. Query filter/sort changes invalidate old cursor.
19. Mutation request timed out after dispatch — no blind retry, goes to `outcome_unknown`.
20. Create Issue response lost but object exists remotely — reconciliation marks success without duplicate.
21. App closes during sync — queue state survives restart.
22. Provider disconnect/revoke — no repeated API calls with invalid token.
23. Manual full sync calculates approximate pages/cost and can be cancelled.
24. Budget guard background share 10% and reserve policy passes tests under different limits/remainings.
25. `/rate_limit` not called on every poll or every application start unless explicitly justified.

## 13. Required operational defaults

- `max_in_flight_per_principal = 1`.
- `background_allowance_formula = min(10% of observed limit, observed remaining minus 20% safety reserve)`.
- `mutative_request_min_spacing = 1s`.
- `background_budget_share = 10%` of observed bucket limit per reset window.
- `primary_safety_reserve = 20%` of observed bucket limit (initial policy; configurable).
- `graphql_default_page_size = 25` for detailed lists; 50 for light summary lists.
- `graphql_max_page_size = 100` (provider rule).
- `graphql_internal_node_budget = 10_000` predicted nodes/query (ForgeDesk policy; not a GitHub limit).
- `secondary_rate_limit_initial_wait = 60s` if no Retry-After and primary remaining is not zero.
- `max_automatic_retries = 3` per classified safe read/background operation; no blind mutation retry.
- active entity sync default: 10–15 minutes only when open/relevant and budget allows.
- selected repository metadata: 30–60 minutes or on-demand.
- historical full sync: manual only.

All defaults must be centralized in configuration with documented bounds. Avoid scattered numeric literals.

## 14. Authentication implementation constraints

- Use a GitHub App, not a distributed private key.
- A native desktop app must never ship the GitHub App private key or client secret.
- Prefer Authorization Code + PKCE if supported by the selected GitHub App auth flow and deployment. Device Flow is acceptable for a constrained desktop client if enabled by app registration and implemented exactly according to GitHub's polling/expiration rules.
- For Device Flow: initial interval from GitHub response; default is documented as 5 seconds; do not poll faster; on `slow_down` add 5 seconds or use returned interval; device/user code default expiration is 15 minutes; honor all server responses.
- User access tokens can expire (documentation describes default 8 hours when expiration is enabled) and refresh tokens may be supplied; handle refresh securely and handle revocation.
- Use least-privilege permissions. Split read permissions and write permissions when possible; explain requested permissions to user.
- A user access token is restricted by the intersection of app permissions and the user's own access. Do not infer repository access from successful OAuth alone; enumerate accessible repositories/installation scope.

## 15. Source documentation / implementation-time verification

Before implementing or releasing GitHub adapter, re-check:
- https://docs.github.com/en/graphql/overview/rate-limits-and-query-limits-for-the-graphql-api
- https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api
- https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api
- https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app
- https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app
- https://docs.github.com/en/apps/creating-github-apps/about-creating-github-apps/best-practices-for-creating-a-github-app

Do not copy a numeric quota into code as an unconditional guarantee. The scheduler uses observed headers and handles the case where limits change.
