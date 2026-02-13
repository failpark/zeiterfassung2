# Feature Research

**Domain:** Professional time tracking for consulting teams
**Researched:** 2026-02-13
**Confidence:** MEDIUM (web access restricted; findings based on existing codebase analysis, legacy UI screenshots, PROJECT.md, and training-data knowledge of the domain — flagged where training-only)

---

## Evidence Base

Primary evidence (HIGH confidence):
- Legacy application screenshots in `docs/` showing actual feature set the team used
- Existing codebase schema (`schema.rs`, `types/index.ts`)
- `PROJECT.md` requirements (validated and active)
- Frontend `TrackingPage.tsx` revealing current UX gaps

Training-data evidence (MEDIUM confidence, flagged):
- Knowledge of Harvest, Toggl, Clockify, and Kimai feature sets from training data

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features the consulting team already relies on or expects as baseline. Missing = product feels broken.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Time entry CRUD (date, begin, end, pause, description) | Core function — without this the product does nothing | LOW | Already exists in Rocket app and React frontend |
| Performed vs billed hours on each entry | Core value proposition — billing depends on it | LOW | Already in schema; current frontend auto-sets billed = performed (bug: should be editable independently) |
| Client selection on time entry | Entries must be attributable to a paying client | LOW | Already exists |
| Project selection (filtered by client) | Work is organized by project within a client | LOW | Already exists; current frontend filters by client correctly |
| Activity type tagging (multi-select) | Consulting teams categorize work type (e.g., travel, email, meeting) | LOW | Already exists as many-to-many; current UI uses checkboxes |
| Sub-project selection (Teilprojekt) | Legacy app had sub-projects; team expects it in rewrite | MEDIUM | NOT in current schema — needs migration; project references visible in legacy screenshots |
| User-scoped entry view | Users should only see their own entries by default | MEDIUM | Current backend has no user filter on `GET /tracking/page` — all users see all entries |
| CRUD for clients | Admin function to manage billing targets | LOW | Already exists |
| CRUD for projects (linked to client) | Admin function to manage project catalog | LOW | Already exists |
| CRUD for activities | Admin function to manage work type taxonomy | LOW | Already exists |
| CRUD for users | Admin can create/manage team members | LOW | Already exists |
| JWT authentication (login/logout) | Multi-user system requires auth | LOW | Already exists |
| Admin vs user role separation | Admins manage reference data; users log time | LOW | Already exists (`sys_role` field); current frontend lacks admin-only UI gating |
| Paginated list of time entries | Large datasets require pagination | LOW | Already exists |
| Entry edit (including billed hours independent of performed) | Billing often differs from raw worked time | MEDIUM | Current frontend hardcodes billed = performed on create AND edit — must expose independent field |
| Entry delete (admin only) | Data correction | LOW | Already exists |
| Copy/Paste entry shortcut | Legacy app had Copy/Paste buttons; consultants re-use similar entries daily | MEDIUM | Not in current app — visible in legacy screenshot action buttons |
| Date-based filtering of entries | Users need to see "what did I log this week?" | MEDIUM | Not in current app — only global pagination |
| Employee filter (admin view) | Admins need to see what each person logged | MEDIUM | Not in current app |

### Differentiators (Competitive Advantage)

Features that make this app worth adopting over a generic tool. Should align with the team's specific consulting workflow.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Calendar week view (Day/Week/Month) | The legacy app had this; it maps to how consultants think about their week — seeing all entries for a week at a glance is far more natural than a flat list | HIGH | Visible in `docs/zeiterfassung_overview.png`: Day/Week/Month toggle, entries laid out on calendar rows with time range and client/project shown inline |
| Travel routes (Fahrstrecken) | Consulting involves client site visits; mileage tracking is needed for expense reimbursement | MEDIUM | Listed in `PROJECT.md` active requirements; visible as "Fahrstrecken" in legacy left-nav; needs its own entity (from/to route with distance or time) |
| Billed hours independent edit with visual diff | When billed != performed, show the delta clearly so users can see what was discounted | LOW | High value for billing review; easy to add once billed field is independently editable |
| Hierarchical summary view (Client > Month > Project) | The legacy app had a drill-down tree: client totals collapsing to month, then to project, with billed and performed columns | HIGH | Visible in `docs/zeiterfassung_tracked_time_order.png` and `zeiterfassung_tracked_time.png` — deferred per PROJECT.md but this IS the reporting feature the team uses for billing |
| Detail list with multi-column rollup (Fakturiert/Aufwand) | Shows individual entries with billed/performed columns side-by-side; supports billing review | MEDIUM | Visible in `docs/zeiterfassung_detail_list.png`; deferred per PROJECT.md |
| Employee overview / Mitarbeiterkalender | Admins can see all employees' calendars side-by-side | HIGH | Visible in legacy left-nav: "Mitarbeiterübersicht" with per-employee sub-nav; useful for capacity planning |
| Per-user daily total display | Shows total hours per day in calendar view, making it easy to spot missing or excessive entries | LOW | Visible in legacy calendar: clock icon + daily total per row |
| Activity token (short code) | Activities have optional short tokens (e.g., "AT" for "Abstimmung telefonisch") used as display abbreviations in tight list views | LOW | Already in schema (`token` field on activity); not surfaced in current UI |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create complexity without proportional value for a small consulting team.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Real-time timer / "start/stop" tracking | Modern tools like Toggl push this | Consultants in meetings log time after the fact, not during; a running timer adds state management complexity and only helps a different workflow | Keep the begin/end/pause form model from the legacy app — accurate and familiar |
| Automatic hour calculation (hide begin/end) | Simpler entry: just enter hours | The team's legacy app and current schema store begin/end times explicitly; these are needed for audit and detailed billing discussions | Calculate performed from begin/end/pause automatically but keep the time fields visible |
| OAuth/SSO | Common enterprise request | Small team doesn't need it; adds dependencies and auth surface area | Email/password with JWT is sufficient — per PROJECT.md "out of scope" |
| Mobile app | Users want to log on the go | Web-first is correct for a desktop-use-case; mobile apps require platform-specific work | Ensure the web UI is responsive enough for occasional mobile use |
| Invoice generation | Natural extension of billing data | Invoicing is a separate business process with legal/format requirements per country; German consulting invoices have specific Vorsteuer/tax requirements | Export CSV or summary data; let accounting software handle actual invoicing |
| Automated email reports | "Send me a monthly summary" | Adds email infrastructure, scheduling, and user preference management | Build the reporting UI first; let users trigger exports manually |
| WebSockets / real-time updates | Multiple users editing simultaneously | Small team; no concurrent editing scenarios worth the complexity | Standard request/response with optimistic UI is sufficient |
| Project budget tracking / burndown | Track hours against budget | Requires budget input, tracking, and alerts — a separate product feature scope | Provides performed vs billed which gives a natural billing ceiling; budget tracking is future scope |
| Billable rates per user/project | Detailed billing calculations | Rate management is complex (per user, per project, per activity, with history); and this team uses billing hours directly, not rate-based | Preserve the explicit billed hours field — the team controls billing decisions manually |

---

## Feature Dependencies

```
[User authentication]
    └──required-by──> [All other features]

[Client CRUD]
    └──required-by──> [Project CRUD]
                          └──required-by──> [Sub-project CRUD]
                                                └──required-by──> [Time entry with sub-project]
                          └──required-by──> [Time entry creation]

[Activity CRUD]
    └──required-by──> [Time entry creation]
                          └──token field──> [Activity abbreviations in list view]

[Time entry CRUD (basic)]
    └──required-by──> [User-scoped entry view]
    └──required-by──> [Calendar week view]
    └──required-by──> [Date-based filtering]
    └──required-by──> [Billed hours independent edit]
    └──required-by──> [Hierarchical summary reports]
    └──required-by──> [Copy/Paste entry]

[User CRUD + admin roles]
    └──required-by──> [Employee filter (admin view)]
    └──required-by──> [Employee calendar overview]

[Travel routes entity]
    └──independent──> separate entity from time entries; parallel track to sub-projects

[Date-based filtering]
    └──required-by──> [Calendar week view]

[Hierarchical summary view]
    └──required-by──> [PDF export] (deferred)
```

### Dependency Notes

- **Sub-projects require Project CRUD:** Sub-projects (Teilprojekt) are children of projects; the project entity must exist and be stable before adding the sub-project layer. Requires a schema migration adding a `sub_project` table with `project_id` FK.
- **Billed hours independent edit is gated on UI fix:** The schema already supports independent performed/billed values. The current frontend hardcodes billed = performed on create and edit. This is a bug fix, not a new feature — must come early.
- **Calendar view requires date filtering:** A calendar week view is just a date-range filter with a specific rendering. The backend endpoint for date-filtered entries must exist before the calendar UI can be built.
- **Reporting/summaries require stable time entry data:** Summary rollups (client > month > project) are derived views over time entries. They should be deferred until the core entry model is stable (including sub-projects).
- **Travel routes conflict with time-entry activities:** "Fahrzeit" is currently an activity type on a tracking entry. Travel routes (Fahrstrecken) are a separate entity. These must not be confused — when adding the travel routes feature, the existing "Fahrzeit" activity type remains for entries where travel is just part of a workday; the travel routes entity handles dedicated route-based mileage records.

---

## MVP Definition

The app is brownfield — the Rocket backend and React frontend already exist. MVP for the rewrite means: everything the team currently has, plus the missing pieces from active requirements.

### Launch With (v1 — Rewrite Core)

- [x] JWT authentication (login, token refresh, logout) — must configure expiration
- [x] CRUD for clients, projects, activities, users
- [x] Time entry CRUD with performed AND billed as independent editable fields — fix current bug where billed is hardcoded to performed
- [x] User-scoped entry view — regular users see only their own entries; admins see all
- [x] Admin role gating in UI — hide admin functions (create/delete) from regular users
- [x] Date-based filtering on time entry list — filter by date range, minimum weekly view
- [x] Activity token display — show short codes in list views where space is tight
- [x] Sub-projects (Teilprojekt) — schema migration + CRUD + selection on time entry form
- [x] Travel routes (Fahrstrecken) — schema + CRUD (from/to + distance or time)

### Add After Validation (v1.x)

- [ ] Calendar week view (Day/Week/Month toggle) — significant UI investment; add once core data model is stable
- [ ] Copy/Paste entry shortcut — reduces friction for daily entry; add when team starts using the system daily
- [ ] Employee filter in admin view — admin needs to review team entries; add when multiple users are active
- [ ] Activity abbreviations in compact list view — useful display polish; add alongside calendar view

### Future Consideration (v2+)

- [ ] Hierarchical summary reports (Client > Month > Project drill-down) — explicitly deferred in PROJECT.md; add once team validates the core workflow
- [ ] PDF export / print — deferred in PROJECT.md; add alongside reporting
- [ ] Employee calendar overview (Mitarbeiterkalender) — high complexity admin view; add with reporting phase
- [ ] Daily total summary in calendar — display polish for calendar view; add with calendar feature
- [ ] Data export (CSV) — useful for handoff to accounting; add with reporting

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Billed hours independent edit (bug fix) | HIGH | LOW | P1 |
| User-scoped entry view | HIGH | LOW | P1 |
| JWT auth with configurable expiration | HIGH | LOW | P1 |
| Admin role gating in UI | HIGH | LOW | P1 |
| Date-based filtering on entries | HIGH | MEDIUM | P1 |
| Sub-projects (Teilprojekt) | HIGH | MEDIUM | P1 |
| Travel routes (Fahrstrecken) | HIGH | MEDIUM | P1 |
| Calendar week view | HIGH | HIGH | P2 |
| Copy/Paste entry | MEDIUM | MEDIUM | P2 |
| Employee filter (admin) | MEDIUM | LOW | P2 |
| Activity token in compact views | LOW | LOW | P2 |
| Hierarchical summary reports | HIGH | HIGH | P3 |
| Employee calendar overview | MEDIUM | HIGH | P3 |
| PDF export / print | MEDIUM | MEDIUM | P3 |
| CSV export | MEDIUM | LOW | P3 |

**Priority key:**
- P1: Must have for launch (rewrite phase)
- P2: Should have, add when core is stable
- P3: Future phase (deferred per PROJECT.md)

---

## Competitor Feature Analysis

Note: Web access was restricted during research. The following is based on training-data knowledge of the professional time tracking market (MEDIUM confidence — verify against current product docs if needed).

| Feature | Harvest | Toggl Track | Kimai (self-hosted) | Our Approach |
|---------|---------|-------------|----------------------|--------------|
| Performed vs billed hours | Billable/non-billable toggle only; no separate "performed" field | Billable toggle only | Has "duration" per entry, no separate billed field | Keep the explicit performed/billed float fields — this is domain-specific to how this team bills |
| Sub-projects | No native sub-projects (tasks within projects only) | No sub-projects | Has "activities" as a project subdivision | Add sub_project entity as a child of project — matches legacy app design |
| Travel/mileage tracking | No native mileage; requires integration | No mileage | Plugin available | Add travel_route entity as first-class feature — important for German consulting |
| Calendar view | No calendar view; list only | No calendar view | Calendar plugin available | Implement week calendar — legacy app had this and it's the team's primary navigation |
| Activity types | "Tasks" per project | Tags | "Activities" per project | Keep global activities (not per-project) — matches existing schema and legacy design |
| Copy entry | No quick copy | "Continue tracking" button | Duplicate entry button | Add Copy/Paste — visible in legacy UI; reduces daily entry friction |
| Hierarchical reports | Client > Project rollup | Basic project summary | Full hierarchical reports | Build in v2 — already designed in legacy app; most important reporting view |
| Multi-user admin | Full team management | Full team management | Full team management | Admin role for user management; user-scoped views are critical |
| Self-hosted | No (SaaS only) | No (SaaS only) | Yes (PHP) | Rust + MariaDB self-hosted — team controls their data |

**Key takeaway:** The big SaaS tools (Harvest, Toggl) lack the specific features this team needs: explicit performed vs billed hours, sub-projects, travel routes, and a calendar view. This is precisely why a custom tool exists. The rewrite must not drop these domain-specific features in favor of generic patterns.

---

## Sources

- Existing codebase: `backend/src/schema.rs`, `frontend/src/types/index.ts`, `frontend/src/pages/TrackingPage.tsx` — HIGH confidence
- Legacy UI screenshots: `docs/zeiterfassung_overview.png`, `docs/zeiterfassung_form.png`, `docs/zeiterfassung_detail_list.png`, `docs/zeiterfassung_tracked_time.png`, `docs/zeiterfassung_tracked_time_order.png` — HIGH confidence
- `.planning/PROJECT.md` requirements — HIGH confidence
- Training-data knowledge of Harvest, Toggl, Clockify, Kimai feature sets — MEDIUM confidence; web access was restricted, verify if needed

---

*Feature research for: Zeiterfassung — professional time tracking for consulting teams*
*Researched: 2026-02-13*
