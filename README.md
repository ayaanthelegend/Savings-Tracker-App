# Savings Tracker 💰

A high-performance, standalone Windows desktop application written in **Rust** using **egui** (via `eframe`). It provides personal finance management, multi-card bank ledgers with tabular running balances, recurring subscription tracking with automated due-date catchup, and custom savings plans with feasibility checking and native Windows toast notifications.

---

## 🎨 Finalized Design & UI Structure

### 1. Left Sidebar Navigation
- **Fixed-width navigation sidebar**: Displays app logo/branding at top (`💰 Savings Tracker`) and navigation items below:
  - `💳 Bank Accounts`
  - `🔄 Subscriptions`
  - `🎯 Savings Plans`
- **Active Navigation Highlighting**: Filled tinted background with vibrant accent border (`#4F7EF8`), not just a color change.
- **Top Bar**: Spans the remaining window width with current section title on the left, live system date (`📅 YYYY-MM-DD`) and Net Worth across all cards on the right.
- **Silent Autosave**: Zero storage-path or autosave text anywhere in the UI. All persistence to `%APPDATA%\SavingsTracker\data.json` is completely silent and atomic.

### 2. Bank Accounts View
- **Card Tabs**: Row of pill-style buttons across top — one per card, with the primary/earning card designated with a `★ [Name] (Primary)` badge, plus a dashed `+ Add card` pill at the end.
- **Summary Panel in One Horizontal Row**:
  - Selected card name + primary badge.
  - Inline stat groups: **Running balance**, **Income this month**, **Spending this month**, **Opening balance**.
  - Right-aligned **Rename**, **Set Primary**, and **Delete Card** controls.
- **Toolbar Row**:
  - `+ Add income` button (tinted emerald/teal).
  - `+ Add expense` button (tinted coral/red).
  - Segmented period filter: `All`, `This month`, `Last 30 days`, `Custom`.
  - Category dropdown.
  - Right-aligned search box.
- **Spreadsheet Transaction Table**:
  - Columns: `Date`, `Description`, `Category`, `In (PKR)`, `Out (PKR)`, `Balance`, `Actions`.
  - Pinned headers & summary: only the table rows scroll in their vertical ScrollArea.
  - All monetary columns are set in a **monospaced numeric font** (tabular figures) for vertical alignment.
  - Distinct Opening Balance anchor row (muted/italic, no actions).
  - Subtle row hover highlight.
  - Auto-generated subscription transactions display an `[Auto]` badge.

### 3. Subscriptions View
- **Summary Cards**: Displays Total Monthly Cost (Prorated), Annual Commitment, and Active/Paused counts.
- **Responsive Card Grid**: Wraps dynamically into 1, 2, 3+ columns depending on window width.
- **Subscription Cards**: Displays Name, Amount + Billing Cycle (e.g. `Rs 8,200 / yr` or `Rs 400 / mo` in monospaced font), and muted meta line (which card it bills, start date, next due date).
- **Dashed `+ Add subscription` Card** at the end of the grid.
- **Automated Catchup**: Missed due dates automatically log "Money Out" transactions in the linked card's ledger and advance due dates.

### 4. Savings Plans View
- **Sub-Tabs**: `🎯 Active Plans` and `📜 History`.
- **Responsive Plan Cards**:
  - Plan name and deadline date at top.
  - Visual progress bar.
  - Meta line showing amount saved vs. target with percentage in monospaced figures.
  - Expandable monthly surplus breakdown.
  - Feasibility calculation warning if required savings exceed monthly income.
- **Dashed `+ New savings plan` Card** at the end of the grid.
- **Muted Plan History Grid**: Completed/expired plans automatically move to History with final completion status.
- **Windows Toast Notification**: Fired on deadline expiration via native Windows Runtime API.

---

## 🚀 Building & Running

### Prerequisites
- Windows 10 or 11
- Rust toolchain (`cargo` and `rustc`)

### Running in Development
```powershell
cargo run
```

### Compiling Distributable Release Binary
```powershell
cargo build --release
```

The resulting standalone executable will be located at:
```
D:\Savings Tracker App\target\release\savings_tracker.exe
```

- **Standalone**: Compiles to a single `.exe` with zero external runtime dependencies.
- **Silent Launch**: Suppresses terminal console via `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`.

---

## 🧪 Automated Test Suite
Run the test suite with:
```powershell
cargo test
```
Verifies:
- Monospaced PKR currency formatting with thousands separators
- Chronological running balance calculation from opening balance
- Subscription cycle advancement and prorated calculations
- Automated missed due date catchup and transaction generation
- Savings plan metrics, feasibility detection, and surplus tracking
- Atomic disk save and recovery
