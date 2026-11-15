# 06. ForgeDesk — обязательная спецификация UI/UX и визуальной реализации

**Версия:** 1.0  
**Статус:** обязательный продуктовый контракт; не набор пожеланий  
**Приоритет:** P0 для Application Shell и Design System; далее P1/P2 по этапам SRS  
**Связанные документы:** `01_PRODUCT_PROJECT.md`, `02_TECHNICAL_SPEC.md`, `04_TOKEN_EFFICIENCY_RULES.md`.

> **Ключевое правило:** этот документ определяет, как ForgeDesk должен выглядеть и вести себя. DeepSeek не имеет права самостоятельно менять информационную архитектуру, основные размеры, компоновку, дизайн-токены, навигацию, плотность или паттерны взаимодействия. Если деталь не описана, нужно использовать существующие токены и компоненты, выбрать минимальное непротиворечивое решение и записать его в `docs/decisions/`. Не изобретать новый визуальный язык.

---

## 1. Цель и характер интерфейса

ForgeDesk — desktop-приложение для ежедневной работы разработчика. Интерфейс должен быть функционально близок к Linear по скорости, плотности и клавиатурному управлению, но иметь собственную визуальную идентичность и более глубокую GitHub-панель.

### 1.1. Обязательные характеристики

1. **Desktop-first.** Основной целевой формат — широкое desktop-окно. Мобильный интерфейс не проектировать.
2. **Высокая информационная плотность.** Списки и доски должны показывать много полезной информации без огромных пустых карточек.
3. **Стабильная геометрия.** Навигация и основные панели не должны прыгать при переходах между разделами.
4. **Предсказуемые взаимодействия.** Одинаковые действия выглядят и работают одинаково в Issues, Projects, PR и других разделах.
5. **Клавиатура — полноценный способ работы.** Любую частую операцию можно выполнить без мыши.
6. **Local-first feedback.** Локальные операции подтверждаются сразу; сетевой статус показывается отдельно и не блокирует интерфейс.
7. **Без декоративного шума.** Нет больших hero-блоков, маркетинговых баннеров, градиентных фонов, декоративных иллюстраций в рабочих представлениях и карточек ради карточек.
8. **AI отсутствует визуально**, пока Optional AI Extension не включён пользователем. Даже место в sidebar под отключённый AI не резервируется.

### 1.2. Чего не делать

- Не делать интерфейс похожим на админ-панель CRUD-шаблона.
- Не делать весь контент набором одинаковых крупных rounded cards.
- Не размещать слева вторую вертикальную панель ради каждого модуля.
- Не прятать основные действия в меню из-за недостатка места.
- Не показывать все поля и метаданные одновременно, если их можно раскрыть по запросу.
- Не создавать новые цвета, размеры, тени, отступы или радиусы вне Design Tokens.
- Не менять названия пунктов навигации, порядок групп или структуру окон без ADR и согласования.
- Не использовать emoji вместо иконок.
- Не добавлять изображения/аватары/графические заглушки, если они не несут информации.

---

## 2. Целевые размеры окна и адаптивное поведение

### 2.1. Целевые viewport

| Сценарий | Размер окна | Ожидаемое поведение |
|---|---:|---|
| Основной reference viewport | 1440 × 900 px | Все главные области доступны; правый detail pane может быть закреплён |
| Компактный desktop | 1280 × 800 px | Sidebar обычной ширины; detail pane открывается по необходимости |
| Минимально поддерживаемый | 1100 × 700 px | Sidebar сворачивается до icon rail; detail pane только overlay |
| Ниже минимума | Менее 1100 × 700 px | Приложение не обязано сохранять полную компоновку; не проектировать mobile-first |

При первом запуске окно открывается с размером около **1440 × 900 px**, если доступная рабочая область позволяет. Размер и состояние окна сохраняются между запусками. Не открывать приложение на весь экран принудительно.

### 2.2. Геометрия основного окна

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Native window title bar (OS-managed; do not replace in v1)                   │
├───────────────────┬──────────────────────────────────────────────────────────┤
│                   │ App header: breadcrumbs / page title / global actions    │ 48 px
│ LEFT SIDEBAR      ├──────────────────────────────────────────────────────────┤
│ 240 px            │                                                          │
│                   │ MAIN CONTENT                                             │
│ grouped nav       │ flexible width                                           │
│                   │                                                          │
│                   │                                                          │
│                   │                                                          │
│                   │                                                          │
│                   │                                                          │
├───────────────────┴──────────────────────────────────────────────────────────┤
│ Status strip: local DB / sync state / background activity (when relevant)     │ 24 px
└──────────────────────────────────────────────────────────────────────────────┘
```

- Использовать системный native title bar Tauri в первой версии. Не реализовывать собственную перетаскиваемую title bar.
- Левая навигация: **240 px** expanded, **64 px** collapsed.
- Верхняя панель приложения: **48 px**.
- Нижняя status strip: **24 px**, не показывать крупные баннеры; при отсутствии событий strip может содержать только компактный local status.
- Основной контент занимает остаток.
- Правая панель деталей не является постоянной третьей колонкой. Она появляется по действию пользователя.

### 2.3. Правая панель деталей (Detail Pane)

- Ширина на широком окне: **440 px**, допустимый диапазон 400–520 px.
- На viewport, где после её открытия основная область стала бы уже 520 px, панель открывается как overlay поверх контента, а не сжимает его.
- Панель закрывается `Esc`, кнопкой Close или повторным выбором текущей сущности.
- Панель не сбрасывает состояние списка, фильтров, сортировки или scroll position.
- Вкладки/секции панели зависят от типа сущности, но общий header, close behavior, actions и scroll pattern одинаковы.
- По умолчанию detail pane закрыта. Открытие панели можно сохранить как пользовательскую настройку, но не включать автоматически при первом запуске.

### 2.4. Sidebar collapse

- В expanded состоянии показываются иконка + текст пункта, названия групп и workspace header.
- В collapsed состоянии остаются иконки; hover/focus показывает tooltip с названием.
- Состояние sidebar сохраняется.
- Нельзя скрывать пункты навигации или менять их порядок при collapse.

---

## 2.5. Reference wireframes (layout contract)

Это схематичные структуры, а не ASCII-art для отображения в приложении. Все области должны совпадать с ранее определённой геометрией.

### My Issues + Detail Pane

```text
┌────────────────────────────┬───────────────────────────────────────────┐
│ SIDEBAR 240                │ HEADER: My Issues       Search  +  Sync   │
│ Workspace / groups         ├───────────────────────────────────────────┤
│                            │ My Issues                New issue        │
│ WORKSPACE                  │ List | Board | Table | Calendar | Timeline│
│ Inbox                      │ Filters  Group  Sort  Saved view          │
│ My Issues                  ├───────────────────────────┬───────────────┤
│  - Pinned View A            │ Issue list                │ DETAIL PANE   │
│  - Pinned View B            │ ID  Title  Status  ...    │ FD-142        │
│ Activity                   │ FD-142 Improve GIF...     │ Title         │
│ Search                     │ FD-141 ...                │ Description   │
│                            │ FD-140 ...                │ Properties    │
│ PLANNING                   │ ...                       │ Relations     │
│ Projects                   │                           │ GitHub links  │
│ Initiatives                │                           │ Activity      │
│ Cycles                     │                           │ Comments      │
│ Roadmap                    │                           │               │
│ Documents                  │                           │               │
│ ...                        │                           │               │
├────────────────────────────┴───────────────────────────┴───────────────┤
│ STATUS STRIP: Local ready · GitHub connected · Up to date               │
└────────────────────────────────────────────────────────────────────────┘
```

### Project Overview

```text
┌────────────────────────────┬───────────────────────────────────────────┐
│ SIDEBAR                    │ HEADER: Projects / EXHI Capture            │
│ (same persistent shell)    ├───────────────────────────────────────────┤
│                            │ [icon] EXHI Capture   Active   ...        │
│                            │ Overview | Issues | Roadmap | Docs | ...  │
│                            ├───────────────────────────┬───────────────┤
│                            │ Description               │ Milestones    │
│                            │ Progress / dates          │ M1 ...        │
│                            │ Issues by status          │ M2 ...        │
│                            │ Active PRs / repositories │ Repositories  │
│                            │ Recent Activity           │ PR summary    │
│                            │                           │               │
└────────────────────────────┴───────────────────────────┴───────────────┘
```

### Pull Request Detail

```text
┌────────────────────────────┬───────────────────────────────────────────┐
│ SIDEBAR                    │ HEADER: Pull Requests / owner/repo#186      │
│                            ├───────────────────────────────────────────┤
│                            │ #186 Improve GIF capture   Open   GitHub ↗ │
│                            │ Overview | Commits | Files | Reviews | ...│
│                            ├───────────────────────────────────────────┤
│                            │ Description / branches / linked issues    │
│                            │ Review summary                            │
│                            │ CI checks                                 │
│                            │ Timeline                                  │
│                            │                                           │
└────────────────────────────┴───────────────────────────────────────────┘
```

### API Monitor

```text
┌────────────────────────────┬───────────────────────────────────────────┐
│ SIDEBAR                    │ HEADER: System / API Monitor               │
│                            ├───────────────────────────────────────────┤
│                            │ Connection · Scheduler · Pause/Resume     │
│                            │ ┌──────────────────┐ ┌──────────────────┐ │
│                            │ │ GraphQL primary  │ │ REST primary     │ │
│                            │ │ Remaining/reset  │ │ Remaining/reset  │ │
│                            │ └──────────────────┘ └──────────────────┘ │
│                            │ Secondary limits / in-flight / reserve    │
│                            │ Usage charts                              │
│                            │ Recent operations table                    │
└────────────────────────────┴───────────────────────────────────────────┘
```

## 3. Design System — точные визуальные правила

Все значения ниже оформить CSS variables/design tokens. Не использовать случайные ad-hoc значения в отдельных компонентах.

### 3.1. Цветовые токены: Dark theme

| Token | HEX | Назначение |
|---|---|---|
| `--fd-bg` | `#0E1117` | Основной фон окна |
| `--fd-sidebar` | `#11151C` | Фон sidebar |
| `--fd-surface` | `#151A22` | Основные поверхности и панели |
| `--fd-surface-raised` | `#1A202B` | Raised surface, меню, popover |
| `--fd-surface-hover` | `#202735` | Hover state |
| `--fd-surface-selected` | `#242D3D` | Selected row/item |
| `--fd-border` | `#293241` | Основные разделители и границы |
| `--fd-border-subtle` | `#202734` | Слабые разделители |
| `--fd-text` | `#E8ECF2` | Основной текст |
| `--fd-text-secondary` | `#A0A9B8` | Вторичный текст |
| `--fd-text-muted` | `#727D8D` | Метаданные и disabled text |
| `--fd-accent` | `#8B7CFF` | Основной акцент, active navigation, focus |
| `--fd-accent-soft` | `#302B52` | Мягкая акцентная подложка |
| `--fd-success` | `#43B581` | Успех, completed, healthy |
| `--fd-warning` | `#E5A84B` | Предупреждение, pending, review |
| `--fd-danger` | `#E16D75` | Ошибка, destructive state |
| `--fd-info` | `#6FA8FF` | Информационный статус |

Цвет семантического статуса всегда сопровождается текстом или иконкой. Нельзя передавать смысл исключительно цветом. Базовое соответствие priority: Urgent → danger, High → warning, Medium → info, Low → muted/secondary, None → без цветного индикатора. Статусы workflow используют категорию (backlog/unstarted/started/completed/canceled) и ограниченную semantic palette; произвольные пользовательские HEX не принимать в v1.

### 3.2. Цветовые токены: Light theme

| Token | HEX | Назначение |
|---|---|---|
| `--fd-bg` | `#F5F6F8` | Основной фон окна |
| `--fd-sidebar` | `#ECEFF3` | Фон sidebar |
| `--fd-surface` | `#FFFFFF` | Основные панели |
| `--fd-surface-raised` | `#FFFFFF` | Меню и popover |
| `--fd-surface-hover` | `#EEF1F5` | Hover state |
| `--fd-surface-selected` | `#E7E9F8` | Selected row/item |
| `--fd-border` | `#D8DDE5` | Основные разделители |
| `--fd-border-subtle` | `#E7EAF0` | Слабые разделители |
| `--fd-text` | `#202633` | Основной текст |
| `--fd-text-secondary` | `#5F6978` | Вторичный текст |
| `--fd-text-muted` | `#8A93A1` | Метаданные и disabled text |
| `--fd-accent` | `#6757D9` | Основной акцент |
| `--fd-accent-soft` | `#EAE7FF` | Мягкая акцентная подложка |
| `--fd-success` | `#23865B` | Успех |
| `--fd-warning` | `#A86A12` | Предупреждение |
| `--fd-danger` | `#C44450` | Ошибка |
| `--fd-info` | `#356FC4` | Информация |

Light theme должна быть полноценной, а не простой инверсией Dark. Контраст текста и границ проверяется отдельно.

### 3.3. Типографика

- Основной шрифт: `Inter`, если доступен в системе; fallback: системный sans-serif (`Segoe UI` на Windows, системный sans-serif на Linux).
- Не скачивать и не включать сторонние font files в репозиторий без отдельного решения.
- Основной размер текста: **13 px**.
- Вторичный текст/метаданные: **12 px**.
- Заголовки секций: **14 px / semibold**.
- Заголовок страницы: **20 px / semibold**.
- Заголовок модального окна: **16 px / semibold**.
- Заголовок Issue в detail pane: **18 px / semibold**.
- Мелкие подписи и timestamps: **11 px**, не использовать для основного контента.
- Line-height основного текста: 1.45–1.55; строки таблиц и списков не раздувать типографикой.

### 3.4. Spacing, размеры, радиусы

Базовая шкала отступов — 4 px:

`4, 8, 12, 16, 20, 24, 32, 40, 48 px`.

| Element | Значение |
|---|---:|
| Основной горизонтальный padding content | 24 px |
| Sidebar horizontal padding | 12 px |
| Page header padding | 24 px horizontal / 16 px vertical |
| Между крупными секциями | 24 px |
| Между контролами формы | 12 px |
| Высота обычной кнопки | 32 px |
| Высота input/select | 32 px |
| Высота компактной строки списка | 36 px |
| Высота обычной строки списка | 40 px |
| Высота пункта sidebar | 32 px |
| Высота toolbar | 40 px |
| Радиус поля/кнопки | 6 px |
| Радиус небольших панелей/menu | 8 px |
| Радиус modal/dialog | 10 px |
| Радиус большой карточки | 10 px maximum |

Не использовать pill-shaped кнопки в качестве общего стиля. Pill допускается только для небольших status/label chips.

### 3.5. Иконки

- Основной набор: Lucide, единая версия/пакет на весь frontend.
- Размер обычной иконки: 16 px; sidebar: 16 px; крупные пустые состояния: максимум 28 px.
- Stroke width единый (около 1.75–2 px).
- Иконки одного назначения должны быть одинаковыми во всех разделах.
- Не использовать emoji, разноцветные emoji или смесь разных icon packs.
- Иконка без подписи допустима только для общепринятых действий (close, search, more, refresh); в остальных случаях нужен tooltip/accessible label.

### 3.6. Тени и motion

- Основная иерархия строится цветом поверхности и тонкими border, а не тенями.
- Тень разрешена для popover, command palette, dialog и overlay panel.
- Не использовать glow, blur-heavy glassmorphism и gradients.
- Hover/focus transition: 100–140 ms.
- Drawer/dialog transition: 140–180 ms.
- Анимация не должна задерживать действие или блокировать ввод.
- Учитывать системную настройку reduced motion.

---

## 4. Информационная архитектура и навигация

### 4.1. Sidebar — фиксированный порядок

Порядок групп и пунктов в первой версии:

```text
WORKSPACE
  Inbox
  My Issues
  Activity
  Search

PLANNING
  Projects
  Initiatives
  Cycles
  Roadmap
  Documents

DEVELOPMENT
  Repositories
  Pull Requests
  Reviews
  CI / Activity

INSIGHTS
  Insights
  Dashboards

SYSTEM
  Integrations
  Automations
  API Monitor

[bottom-pinned]
  Settings
  Help / About
```

- Workspace header находится над группами: слева простой mark на основе Lucide `layers` (18 px), рядом текстовый wordmark `ForgeDesk` и компактное имя workspace. Не создавать сложный кастомный логотип в v1. Dropdown workspace в single-workspace v1 не должен создавать фиктивное переключение организаций.
- Группы сворачиваются независимо; collapsed state сохраняется.
- Active item имеет мягкую accent background, accent icon/text и неяркую левую marker line (2 px).
- Пункты с количеством непрочитанных событий могут показывать маленький count badge. Не показывать нулевые счётчики.
- `My Issues` имеет раскрываемую область `Pinned Views`: пользователь может закрепить до 5 Saved Views; остальные доступны через `Manage views`. Эти ссылки не должны менять фиксированный порядок основных nav groups.
- Settings и Help/About закреплены в нижней части.
- AI Assistant добавляется только при активном AI Extension; размещается отдельным пунктом в группе SYSTEM после Integrations. В выключенном состоянии не существует в DOM/nav model.

### 4.2. Верхняя панель (App Header)

Слева направо:
1. Breadcrumbs (до двух уровней; длинная цепочка сокращается).
2. Название текущей страницы, если оно не очевидно из breadcrumb.
3. Flexible spacer.
4. Global search trigger (`Ctrl+K`/`Cmd+K`).
5. Quick create (`+`, `C` shortcut).
6. Activity/Inbox indicator, только если есть непрочитанное.
7. Sync state compact indicator.
8. User-local menu/settings entry (без avatar fake user).

Header не должен содержать дублирующую строку навигации.

### 4.3. Нижняя status strip

Показывает только состояние, полезное в текущий момент:
- `Local database ready` / error;
- `Offline` / `Online`;
- `Sync paused` / `Syncing 3 items` / `Up to date` / `Pending sync: N`;
- по клику — краткая панель последней активности синхронизации.

Не показывать декоративный постоянный spinner. Во время отсутствия активных задач strip остаётся спокойной и компактной.

---

## 5. Общие паттерны взаимодействия

### 5.1. Page layout

Стандартная страница имеет:
1. App Header (глобальный, 48 px).
2. Page heading: title, optional description, primary action.
3. Toolbar: view switcher, filters, grouping, sort, search within view, more actions.
4. Main content.

Не добавлять hero section. Заголовок страницы начинается в 24 px от края content region. Основной контент не должен растягивать строки текста на всю ширину, если это мешает чтению; для редакторов документов допускается ограничение ширины текста.

### 5.2. Buttons

Только четыре визуальных типа:
- **Primary:** одно основное действие в конкретном контексте; accent fill.
- **Secondary:** обычное действие; surface + border.
- **Ghost:** toolbar/row actions; прозрачный фон.
- **Destructive:** удаление/опасное действие; danger color и подтверждение.

Высота стандартной кнопки 32 px, small 28 px. В одной области страницы не размещать более одной dominant primary action, если нет отдельного понятного workflow.

### 5.3. Forms

- Label над полем, helper/error text под полем.
- Required marker только для действительно обязательных полей.
- Формы не должны занимать весь экран для простого создания Issue.
- Для создания Issue использовать compact modal/popover, а для сложного редактирования — detail pane.
- Ввод автоматически сохраняется только там, где это безопасно и явно видно; в остальных случаях Save/Cancel.
- Ошибки валидации показывать у конкретного поля и в общей форме при необходимости.

### 5.4. Menus, popovers, dialogs

- Context menus открываются у соответствующего объекта и не меняют расположение при повторении.
- Popover закрывается Escape и кликом вне.
- Dialog имеет явный title и primary/secondary action в нижней области.
- Destructive dialog описывает конкретный объект и последствия.
- Нельзя использовать browser `alert/confirm/prompt`.

### 5.5. Loading, empty, error, offline

Каждый data view реализует все состояния:
- **Loading:** skeleton повторяет форму реальных строк/панелей; не использовать полноэкранный spinner для локальных данных.
- **Empty:** краткое объяснение и одно contextual action; без маркетинговой иллюстрации.
- **Error:** понятное сообщение, Retry если действие безопасно, технические подробности под раскрытием.
- **Offline:** локальные данные отображаются; remote status явно отмечен как stale/offline.
- **Stale:** показывать `Last updated`/`Data through` и не выдавать кэш за live data.
- **Pending sync:** показывать локально сохранённое изменение с pending marker, не блокировать дальнейшую работу.

### 5.6. Toasts

- Появляются справа снизу над status strip.
- Информируют об успехе/ошибке/предупреждении, не дублируя постоянную историю.
- Автоматически исчезают через 3–5 секунд, ошибки с требуемым действием не исчезают автоматически.
- Toast не является единственным местом отображения критической ошибки; она остаётся в сущности/Inbox/Activity.

### 5.7. Selection и bulk actions

- Single click выбирает строку/карточку; открытие detail pane — по клику на title или Enter.
- Checkbox selection независим от открытия detail pane.
- При выбранных элементах появляется компактная bulk action toolbar над списком.
- `Esc` снимает selection, если нет открытого dialog.
- Bulk operations должны явно показывать количество выбранных сущностей и быть обратимыми либо подтверждаться.

---

## 6. Спецификация ключевых экранов

Для каждого экрана ниже описаны обязательные блоки. DeepSeek может выбирать внутренний способ реализации, но не менять видимую структуру и поведение без ADR.

### 6.0. First Run, onboarding and default route

**First launch:**
1. Показывается компактный onboarding dialog/window, а не публичная landing page.
2. Шаг 1: создать локальное workspace. Поле имени имеет default `My Workspace`; пользователь может изменить его.
3. Шаг 2: выбрать locale (`System`, `English`, `Русский`) и theme (`Dark`, `Light`, `System`). Defaults: System locale (fallback English), Dark theme.
4. Шаг 3: предложить подключить GitHub как optional action (`Connect GitHub` / `Skip for now`). GitHub не обязателен для завершения onboarding. GitLab/Gitea подключаются позже через Integrations.
5. После onboarding открыть `My Issues`. Если задач нет — показать compact empty state с `Create your first issue` и вторичным `Create a project`.

**Subsequent launch:** открыть последнюю страницу, если она существует и доступна; иначе `My Issues`. Не открывать автоматически модальный диалог подключения GitHub. Восстанавливать размер окна, sidebar state и тему.

**Workspace assumptions:** в v1 workspace один, локальный и single-user. UI не должен создавать экран выбора организации/пользователя или фальшивое account avatar menu.

### 6.1. Inbox

**Назначение:** единый список новых/изменившихся событий.

**Layout:**
- Page heading `Inbox`, справа `Mark all read` и menu.
- Под header — компактные tabs/filters: `All`, `Unread`, `GitHub`, `Local`, `Sync`.
- Основная область — вертикальный список событий с группировкой по `Today`, `Yesterday`, `Earlier`.
- Каждая строка: event icon, title/summary, entity key, source badge, relative timestamp, unread marker.
- Click по строке открывает связанный объект в Detail Pane.
- В row menu: Mark read/unread, Archive, Open source (если есть URL).
- Пустое состояние: `You're all caught up` + ссылка `View activity`.

**Не делать:** карточную плитку событий, отдельную dashboard-страницу с графиками в Inbox, огромные иконки.

### 6.1.1. Search

- Dedicated Search page имеет поле ввода в верхней части (ширина до 720 px), tabs: `All`, `Issues`, `Projects`, `Documents`, `Repositories`, `Pull Requests`, `Commits`.
- По мере ввода ищется локальный индекс; query debounce 150–250 ms.
- Строка результата: type icon, title/identifier, one-line context/path, source badge, updated timestamp.
- Recent searches доступны только локально и очищаются из Settings.
- Provider-wide remote search — отдельная кнопка/action `Search GitHub`, с предупреждением, что операция расходует API budget; не выполнять автоматически.
- Empty state до ввода показывает recent searches и короткую подсказку по синтаксису, если он реализован.

### 6.2. My Issues

**Назначение:** основной рабочий список задач пользователя.

**Default view:** List.

**Toolbar:** view selector (List/Board/Table/Calendar/Timeline — доступные режимы по этапу), filter, group, sort, search in view, saved view menu.

**List columns (default):**
- Checkbox;
- Issue ID (FD-123 или remote number);
- Title;
- Status;
- Priority;
- Project;
- Labels (до двух видимых + `+N`);
- Updated (relative date).

Строка default 40 px, compact 36 px. Заголовки колонок 12 px secondary. Колонки можно менять местами, скрывать/показывать и менять ширину; Title имеет flexible width и не может быть уже 240 px, если общий viewport это позволяет.

Row hover показывает компактные actions справа; не показывать их постоянно.

**Board mode:**
- Колонки — workflow statuses, сгруппированные по category только если пользователь выбрал такую группировку.
- Column header содержит status name + count.
- Card: ID, title (до 2 строк), priority icon, до 2 labels, project/estimate в нижней строке при наличии.
- Drag-and-drop изменяет status; после optimistic update показывать pending sync, если это связано с remote.
- WIP limit визуально предупреждает, но не блокирует перемещение без явного правила.

**Table mode:** spreadsheet-like data table with the same entity/filters as List; resizable/reorderable columns, inline editable only for safe local fields, sticky header, horizontal scroll inside table only. Default columns match List. Row height remains 36/40 px.

**Calendar mode:** month view by default, with week toggle. Issues appear as compact event bars based on due date; optional start-date range visualization. Clicking an item opens Detail Pane. Dragging an item to a different date is disabled until explicit edit mode and must show a confirmation/undo path when it changes a remote-owned field.

**Filters:** раскрываемая filter builder panel шириной до 420 px; строка условия состоит из field selector, operator selector и value control. Группы AND/OR визуально вложены на один уровень; глубже одного уровня в v1 не требуется. `Add filter` и `Add group` находятся внизу панели. Активные filters отображаются chips под toolbar и могут удаляться по одному. Условие с некорректным/устаревшим полем показывает inline validation, а не тихо исключается.

**Detail Pane:** открывается на выбранном Issue; список не теряет позицию.

### 6.3. Issue Detail Pane

**Header:**
- Issue key;
- source/provider indicator;
- status dropdown;
- overflow menu;
- close.

**Body, в указанном порядке:**
1. Editable title (18 px semibold).
2. Description Markdown preview/editor.
3. Properties grid: Status, Priority, Type, Project, Cycle, Milestone, Estimate, Start/Due date, Labels.
4. Parent/sub-issues block (hierarchy, progress if relevant).
5. Relations block (`Blocks`, `Blocked by`, `Related`, `Duplicate`).
6. Linked GitHub/GitLab/Gitea entities: Issue/PR/branch/commit references.
7. Activity/Comments timeline, newest at bottom for comments; history events chronological and compact.
8. Attachments.

Секции сворачиваемые, их state сохраняется локально. Properties можно редактировать inline. Empty relation blocks показывают компактный `Add relation`, а не большой empty card.

При изменении remote-owned field отображать source/ownership indicator и не создавать иллюзию, что локальная правка уже ушла в GitHub.

### 6.3.1. Create Issue dialog

- Dialog width: 720 px max; on 1100 px viewport width 640 px max.
- Header: `New issue`; top-right close.
- Required title field at top, autofocus.
- Description Markdown textarea below; collapsed/short by default but expandable.
- Compact property row: Status, Priority, Type, Project.
- Secondary fields behind `More properties`: Cycle, Milestone, Estimate, Due date, Labels, Parent issue.
- Source defaults to `Local`; external provider publication/link is a separate explicit action and is never selected implicitly.
- Footer: `Create issue` primary and `Cancel` secondary; keyboard submit via Ctrl+Enter.
- On success: persist locally, close dialog, show toast, insert/select new issue in current view and open Detail Pane only if user setting says so (default: open detail pane).
- Validation: title required and trimmed; duplicate title is allowed.

### 6.4. Projects — list

- Page title `Projects`, primary action `New project`.
- Toolbar: status, initiative, priority, date range, archived toggle, search.
- Default view — плотная таблица/list, не крупные плитки.
- Default columns: Name, Status, Progress, Priority, Target date, Repositories, Updated.
- Project row click открывает Project Overview; ID/related issue counts — secondary metadata.
- Optional grid view разрешён только как альтернативный view, не default.

### 6.4.1. Create Project dialog

- Dialog width 560 px.
- Required project name; optional description.
- Fields: status, priority, start date, target date, Initiative (optional).
- Repository association is configured after creation from Project Overview; do not force GitHub connection in this dialog.
- Create button saves locally and navigates to Project Overview.

### 6.5. Project Overview

**Project header:** icon/color marker, project name, status, priority, date range, `...` menu; primary action `Add issue`.

**Tabs in fixed order:** `Overview`, `Issues`, `Roadmap`, `Documents`, `Repositories`, `Activity`.

**Overview blocks:**
- short description;
- progress bar + explicit progress method;
- Status and dates;
- Milestones list/timeline;
- Active issues summary (counts by status);
- Linked repositories and open PR summary;
- Recent activity.

Blocks are compact and aligned to a two-column grid when width > 1200 px; one column otherwise. Do not render every block as a big card. Use section heading + divider.

### 6.6. Initiatives

- List/table default, rows show name, status, progress, target date and number of linked projects.
- Initiative details use same heading pattern as Project.
- Initiative Overview shows objective, dates, linked Projects and aggregate progress.
- Clicking project opens Project Overview; not a modal chain.
- Do not introduce company/team/owner concepts into the single-user UI.

### 6.7. Cycles

- Page header with cycle selector and date range.
- Main content split: active cycle summary + issues list/board.
- Summary: start/end, days remaining, total estimate, completed estimate, issue counts, carry-over count.
- Below: cycle issues with same list row component as My Issues.
- Analytics tab/section: burnup/burndown and velocity history.
- No sprint-planning jargon that implies a team; copy should say `Cycle` / `Work cycle`.

### 6.8. Roadmap / Timeline

- Horizontal time axis at top; rows represent Initiatives/Projects/Milestones depending on selected level.
- Left fixed label column, timeline area horizontally scrollable.
- Today marker always visible when within date range.
- Bars show start/end dates, status color at low saturation, progress fill and label.
- Dependency lines are optional overlay, toggled explicitly; do not draw dense spaghetti by default.
- Dragging dates disabled by default until edit mode is activated; keyboard accessible date editing exists.
- Zoom presets: 2 weeks, 1 month, 3 months, 6 months, 1 year.

### 6.9. Documents

- Left document tree/list pane inside content area (not global sidebar), center editor, optional right outline panel.
- Markdown source/preview or rich Markdown editor; keyboard shortcuts and code fences preserved.
- Document title at top; breadcrumb/related project metadata below.
- Autosave with visible `Saved` / `Saving` / `Offline changes` state.
- Search documents globally and within current document.
- Export/Import Markdown in document menu.
- Do not implement a full Notion-like block editor; Markdown semantics and file portability are priorities.

### 6.10. Repositories

- Default list/table with provider icon, `owner/repo`, description, visibility, default branch, local path status, latest sync time, project links.
- Repository detail page tabs: `Overview`, `Issues`, `Pull Requests`, `Branches`, `Commits`, `Releases`, `Settings` (only where supported).
- Overview shows local clone/path state, remote URL, default branch, open issue/PR counts, last fetch/sync, linked projects.
- Do not fetch every branch, commit, file or PR detail just to render repository list.
- Unlinked local repository state and unselected remote repositories must be visibly distinct.

### 6.11. Pull Requests

- Default table/list. Columns: PR, Repository, State, Draft, Review, Checks, Source branch → target branch, Updated.
- Filters: provider, repo, state, draft, review state, check conclusion, linked/unlinked issue.
- PR Detail Pane header: `owner/repo#number`, title, state badge, external link, close.
- Sections/tabs in fixed order: `Overview`, `Commits`, `Files`, `Reviews`, `Checks`, `Timeline`.
- Overview: description, branches, author, reviewers, linked issues, mergeability/state, labels.
- Files tab loads changed files/diff only when opened; show file tree left and diff center. Support Unified and Split view. Diff must be readable; long files virtualized/loaded progressively.
- Reviews tab shows review summary and inline comments. Review actions (Approve/Request changes/Comment) are explicit buttons and must be real provider operations.
- Checks tab shows check name, status, conclusion, duration, details URL and refreshed timestamp.
- Merge button only visible when provider permissions and state allow. Confirm merge and show target branch; never auto-merge.

### 6.12. Reviews

- Dedicated inbox-like queue with tabs `Requested`, `Created by me`, `Participating`, `Completed`.
- Each row: repository/PR, title, author, requested reviewer, review state, CI state, age.
- Clicking opens PR Detail Pane directly on Reviews or Overview as appropriate.
- No duplicate independent PR detail UI; reuse PR components.

### 6.13. CI / Activity

- Timeline/list of CI workflow runs and check runs.
- Filters: repository, branch, status/conclusion, date range.
- Row: workflow/check name, repo, branch/commit, status, duration, started/finished time.
- Detail pane: jobs/steps where provider API supports, error summary, logs link; do not download full logs automatically.
- Local app events and provider CI events must be visually distinguishable by source marker.

### 6.14. Insights

- Analytics page with metric selector and date-range selector.
- Default view: small metric summary row followed by one or two large charts; no wall of charts.
- Every chart shows metric name, period, sample/data source, `Data through <timestamp>` and incomplete-data warning when applicable.
- Chart tooltips show exact values and dates.
- Filters: project, repository, cycle, issue type, date range.
- Charts use the same semantic palette; never invent a color per chart series without stable mapping.

### 6.15. Dashboards

- Dashboard canvas with explicit `Edit dashboard` mode.
- Add widget menu; supported initial widget types: metric card, line chart, bar chart, pie/donut, issue list, PR list, cycle summary, API budget summary.
- Grid is 12 columns; widget widths 3/4/6/8/12 columns; saved layout persists.
- Widgets have title, configuration, source timestamp and overflow menu.
- In view mode no drag handles; edit mode shows resize/move affordances.
- Do not implement arbitrary scripting/custom HTML widgets.

### 6.16. Integrations

- Provider cards/list with connection state, account/host, granted permissions, selected repo count, last successful sync.
- GitHub is shown first and designated primary provider; GitLab and Gitea appear as supported secondary providers.
- Actions: Connect, Reauthorize, Configure repositories, Sync now, Disconnect.
- Permission list must be readable and explain read/write scope separately.
- Disconnect requires confirmation and explains local cached data retention policy.

### 6.17. API Monitor

This is a first-class screen and must not be reduced to a tiny number in the header.

**Top summary:** connection, last observation, next reset time, scheduler state, manual pause/resume.

**Budget panels:**
- GraphQL primary points: limit, remaining, used, observed at, reset at, actual ForgeDesk-observed consumption since reset;
- REST primary requests: same;
- REST resource buckets (core/search/code_search/etc.) as observed;
- secondary pause state: active/clear, reason, retry-after/reset timing;
- conservative internal budget threshold and current background reserve;
- concurrency and dispatcher state (normally `1 in-flight`).

**Charts/tables:**
- API requests/cost over time, grouped by operation class;
- GraphQL actual cost per operation;
- REST conditional request counts: 200 vs 304;
- requests by priority: interactive, foreground refresh, background sync, manual full sync, reconciliation;
- last N sanitized operations: time, endpoint/operation label, family, cost/count, latency, status, retry count.

**Important labels:** distinguish `GitHub reported remaining` from `ForgeDesk observed usage`. Never imply the local app knows exact quota consumed by other apps between observations.

**Actions:** pause/resume background sync, lower/raise local background budget within safe documented bounds, retry a safe failed read, open diagnostics. No button may blindly replay an uncertain mutation.

### 6.18. Automations

- List of rules with enabled toggle, trigger summary, last run, last result.
- Rule editor is a three-part vertical flow: `When` → `If` → `Then`.
- Trigger and action choices are grouped by Local, GitHub, GitLab, Gitea.
- Each rule has test/preview mode against sample event when feasible.
- Run history table shows event, action, result, time, dedupe key summary.
- Remote write actions carry explicit permission/confirmation settings.
- No visual node graph in v1; use clear form builder to avoid complexity.

### 6.19. Settings

Settings use a secondary left sub-navigation within the content region; main global sidebar remains visible.

Fixed groups:
- General (language, date/time, startup, window behavior);
- Appearance (theme, density, accent only if supported by token system);
- Workspace (ID prefix, defaults, workflow);
- Storage (database path readonly/display, attachments, backup/restore);
- Git (executable detection, terminal/IDE preferences, default branch naming pattern);
- Integrations (providers, credentials status and permissions);
- Sync (intervals, budgets, pause policies);
- Keyboard shortcuts;
- Advanced (logs, diagnostics, reset options).

Do not expose arbitrary hidden developer settings as user options. Dangerous reset/delete actions require typed/explicit confirmation and a backup recommendation.

---

## 7. Command Palette, Search and Keyboard Contract

### 7.1. Command Palette

- Open with `Ctrl+K` on Windows/Linux; support `Cmd+K` if later macOS target is added.
- Width: 640 px max; centered near upper-middle (about 35% viewport height), max height 520 px.
- Search input autofocuses; results grouped by `Actions`, `Issues`, `Projects`, `Repositories`, `Pull Requests`, `Views`.
- Recent/frequently used commands shown when query is empty.
- Arrow keys navigate; Enter executes; Esc closes; Tab may move to secondary action only if explicit.
- Search results use icon, primary label, secondary path/key and optional shortcut.
- Results are local-first. Remote result search must use local cache or an explicit provider search action, not fire network requests on every keystroke.

### 7.2. Keyboard shortcuts

| Shortcut (Win/Linux) | Action |
|---|---|
| `Ctrl+K` | Open Command Palette / global search |
| `C` | Create Issue (when focus is not in text input) |
| `N` | Create item in current context where unambiguous |
| `G` then `I` | Go to Inbox |
| `G` then `M` | Go to My Issues |
| `G` then `P` | Go to Projects |
| `G` then `R` | Go to Repositories |
| `Esc` | Close topmost popover/dialog/detail pane or clear selection in that order |
| `Enter` | Open selected row / confirm focused action |
| `Space` | Toggle checkbox when focused |
| `Ctrl+Enter` | Submit form/comment where applicable |
| `?` | Show keyboard shortcut help |

- Shortcut hints appear in menus/tooltips.
- No shortcut may intercept typing while an input, textarea, editor or contenteditable has focus, except documented editor shortcuts.
- Chord shortcuts (e.g., `G` then `I`) have a visible short timeout and cancellation via Esc.

### 7.3. Search

Global search searches local cache first: Issues, Projects, Initiatives, Documents, repositories, PR cache, commits as available. Search result indicates entity type and source. Search must not synchronously call GitHub for every keystroke. Provider search is a separate explicitly initiated action and obeys API budgets.

---

## 8. Localization, accessibility and content rules

### 8.1. Localization

- UI strings must be externalized through an i18n layer from the first implementation; do not hardcode user-visible copy in components.
- Ship `en` and `ru` locales in the first UI release.
- Default locale follows OS locale if supported; fallback to English.
- Dates/numbers follow selected locale and system timezone; data stored in UTC.
- Short labels must fit at the specified widths; allow text expansion and tooltip for truncation.
- Technical identifiers, repository paths, branch names, Git commands and code remain unlocalized.

### 8.2. Accessibility

- Full keyboard navigation and visible focus states.
- Focus ring uses accent token and must not be removed.
- Buttons/icon-only controls have accessible names.
- Dialog traps focus and restores it to the opener on close.
- Escape closes only the topmost layer.
- Contrast should meet WCAG 2.1 AA for normal text where practicable; semantic status cannot rely on color alone.
- Respect OS reduced motion.
- Drag-and-drop actions have keyboard alternative.
- Do not use tiny text below 11 px for necessary information.

### 8.3. Content style

- Concise, neutral, developer-oriented.
- Use consistent terminology: `Issue`, `Project`, `Initiative`, `Cycle`, `Repository`, `Pull Request`, `Review`, `Check`, `Sync`.
- Avoid team/assignee/organization copy that implies multi-user collaboration. The local owner is not displayed as a fake avatar/user.
- Errors explain what happened, whether data was saved locally, whether remote state is known, and what safe next action exists.

---

## 9. Component inventory (required shared components)

Implement reusable components rather than individual one-off variants:

- `AppShell`, `AppSidebar`, `SidebarGroup`, `SidebarNavItem`;
- `AppHeader`, `Breadcrumbs`, `StatusStrip`;
- `PageHeader`, `PageToolbar`, `ViewSwitcher`;
- `IssueList`, `IssueRow`, `IssueBoard`, `IssueCard`, `IssueDetailPane`;
- `EntityDetailPane` base behavior;
- `StatusBadge`, `PriorityIndicator`, `LabelChip`, `ProviderBadge`, `SyncStateBadge`;
- `FilterBuilder`, `ActiveFilterChip`, `SortMenu`, `GroupMenu`;
- `CommandPalette`, `GlobalSearch`;
- `Modal`, `ConfirmDialog`, `Popover`, `ContextMenu`, `Tooltip`, `Toast`;
- `EmptyState`, `LoadingSkeleton`, `ErrorState`, `OfflineBanner` (compact inline form, not large banner);
- `RepositoryPicker`, `ProviderConnectionCard`;
- `ApiBudgetPanel`, `ApiRequestTable`;
- `MarkdownEditor`, `MarkdownPreview`;
- `ChartPanel`, `MetricCard`;
- `DataTable` with column configuration and virtualization.

Shared components must use design tokens. A component may have variants only when their purpose is documented. Do not create `Button2`, `NewCard`, `ModernInput` duplicates to work around the design system.

---

## 10. Frontend implementation constraints

1. CSS variables are the source of truth for all colors, sizes, spacing, radii and typography values.
2. Build a small component layer on accessible primitives. Do not install a full UI kit and accept its default styling.
3. No inline arbitrary hex colors in feature components; use tokens. Exception: data visualization series mapping must still be centralized in chart tokens.
4. No one-off layout rules that violate the window geometry specified above.
5. Use the same components for repeated Issue rows/cards across My Issues, Project Issues, Cycle Issues and linked Issues.
6. Keep view state (filters, sort, scroll, selected item, open panel) stable during background sync.
7. Use virtualization for lists beyond 100 rows or where performance profiling shows need; do not virtualize short static forms.
8. Use optimistic updates only for local writes. Remote writes must show queued/pending/confirmed/failed state and must not be represented as confirmed before provider response.
9. Error and empty states are part of each feature's implementation, not a later polish task.
10. The UI must never block on network calls. All provider operations are async and have cancellation/timeout behavior where safe.
11. Do not show AI navigation/actions if extension is disabled or absent.
12. Avoid arbitrary resizing of columns/panes that can make the layout unusable; persist sensible bounds.

---

## 11. Visual references and interpretation

Use the following products only as **interaction and information-density references**, not as pixel-perfect sources or permission to copy branding:

- **Linear:** compact issue list, status/priority presentation, sidebar organization, command palette, issue detail panel, saved views and project navigation.
- **GitHub Pull Request UI:** source/target branch presentation, check statuses, review summary, changed-file navigation and inline diff/comment model.
- **Raycast:** focused command palette behavior, keyboard-first search and concise grouped actions.
- **Notion/Obsidian (limited reference):** readable Markdown/document editing and document navigation only; do not copy block-editor complexity.

Interpretation rules:
- Keep Linear-like density and keyboard speed.
- Keep GitHub-native technical detail in PR/review screens.
- Keep ForgeDesk's graphite/violet visual identity from Section 3.
- Do not copy logos, exact branded icons, proprietary illustrations or pixel layouts.
- If a screenshot/reference conflicts with the exact geometry and tokens in this document, this document wins.

---

## 12. UI delivery sequence and visual gates

UI is not a late cosmetic pass. The visual framework is implemented before feature screens.

### UI Gate 0 — Design System + App Shell (Stage 1)

Deliver:
- app window and native title bar;
- exact shell geometry;
- expanded/collapsed sidebar;
- app header and status strip;
- dark and light themes with all tokens;
- typography, buttons, inputs, menus, dialogs, tooltips, toasts;
- keyboard focus and navigation;
- locale switch / OS locale detection;
- placeholder pages using the correct page template.

**Acceptance:** at 1440×900 and 1280×800 screenshots, geometry and token values match this specification. No generic template dashboard. Screenshot set: dark/light × sidebar expanded/collapsed.

### UI Gate 1 — Issues and Views (Stage 2)

Deliver:
- My Issues List;
- Board;
- filters and saved views;
- Issue Detail Pane;
- create/edit Issue forms;
- command palette;
- loading/empty/error/offline states.

**Acceptance:** visual and interaction review at 1440×900; list/detail state persists; all common actions reachable by mouse and keyboard.

### UI Gate 2 — Projects/Planning/Documents (Stage 3)

Deliver Projects list/overview, Initiatives, Cycles, Roadmap and Documents as specified above.

### UI Gate 3 — GitHub screens and API Monitor (Stages 4–6)

Deliver Repositories, Pull Requests, PR Detail, Reviews, CI/Activity, Integrations and fully detailed API Monitor. Provider data states must distinguish cache/live/pending/offline.

### UI Gate 4 — Inbox, Automations, Analytics and Dashboards (Stage 7)

Deliver event-driven screens and configurable dashboards.

### UI Gate 5 — Final polish (Stage 10)

Keyboard audit, accessibility, empty/error/offline states, light theme parity, Windows/Linux screenshot regression and performance.

---

## 13. Screenshot-based visual acceptance

For each UI Gate, capture screenshots at fixed viewport sizes and theme states. Keep reference artifacts in `docs/ui/reference/` and implementation captures in `docs/ui/current/` (do not commit huge raw debug screenshots; optimized PNG/WebP is sufficient).

Required baseline captures:

1. App shell, dark, 1440×900, sidebar expanded.
2. App shell, light, 1440×900, sidebar expanded.
3. App shell, dark, 1280×800, sidebar collapsed.
4. My Issues List with at least 12 rows and Issue Detail Pane open.
5. Board with at least 4 status columns and 3 cards per column.
6. Project Overview with milestones, issues and repository links.
7. Pull Request Detail with Overview/Checks/Files examples.
8. API Monitor showing GraphQL/REST primary budgets, secondary state and operation history.
9. Command Palette open with grouped search results.
10. Dialog, menu, empty state, error state and offline state.

Visual acceptance checks:
- Sidebar width, header height, pane geometry and content padding match the documented dimensions within ±2 px at reference viewport.
- Colors come from tokens; no unexplained colors.
- Typography sizes match tokens; text is not clipped or unreadably small.
- Row/card density matches specifications.
- No horizontal overflow at supported minimum viewport.
- No placeholder lorem ipsum, decorative fake charts or fake GitHub content in a release build.
- UI does not jump when background sync updates data.
- Focus, hover, selected and disabled states are visibly distinct.
- All screens have loading/empty/error/offline states.

Before declaring a UI Gate complete, DeepSeek must compare screenshots with this document and report each check as PASS/PARTIAL/FAIL. If screenshot tooling is unavailable in the environment, do not claim visual acceptance; report it as SKIPPED and provide reproducible manual capture instructions.

---

## 14. No-autonomy rule for visual design

DeepSeek must not independently:
- change the app name/logo placement or sidebar order;
- replace the graphite/violet design direction;
- change the shell from sidebar + content + optional right detail pane;
- choose a card-first dashboard layout as the default;
- add an always-visible AI panel;
- replace List/Board with a different primary navigation model;
- add mobile-first navigation or responsive behavior outside this contract;
- change typography scale, spacing scale, standard row height, window minimum or pane sizes;
- substitute a generic component library theme;
- invent new major screens or remove required screens;
- mark a screen complete based only on a compiling frontend.

When a genuine usability issue requires a deviation:
1. explain the observed problem;
2. provide the smallest proposed change;
3. record an ADR/UI decision with before/after behavior;
4. preserve existing tokens and visual language;
5. continue with the current spec until the user approves the change if it materially changes the interface.

Minor implementation details (component filenames, internal state management, accessible primitive selection) may be chosen autonomously as long as the visible result matches this document.

---

## 15. UI Definition of Done

A screen is complete only when all applicable items pass:

- [ ] Uses AppShell/PageHeader/PageToolbar and shared tokens.
- [ ] Matches specified layout and screen composition.
- [ ] Has dark and light theme support.
- [ ] Has localized strings (English and Russian).
- [ ] Supports keyboard navigation and visible focus.
- [ ] Has loading, empty, error, stale/offline and pending states as appropriate.
- [ ] Does not block on network; preserves local interaction.
- [ ] Preserves scroll, filters and selection during background updates.
- [ ] Reuses shared components rather than duplicating visual patterns.
- [ ] Has functional interactions; no inert controls or fake data presented as live.
- [ ] Has screenshot artifacts and visual gate report.
- [ ] Has frontend tests for the main user flow and relevant component states.

A screen that merely renders or compiles is **not** considered complete.
