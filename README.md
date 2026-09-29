# Savings Tracker 💰

A high-performance, standalone Windows desktop application written in **Rust** using **egui** (via `eframe`). It provides comprehensive personal finance management, multi-card bank ledgers with running balances, recurring subscription tracking with automated catchup, and goal-oriented savings plans with live progress tracking and native Windows toast notifications.

---

## 🌟 Key Features

### 1. Bank Account & Card Tracker

- **Multi-Card / Account Management**: Add, rename, and delete cards across tabs.
- **Primary / Earning Account**: Distinguished by `★ Primary (Earning)` badge (where income normally lands, while other cards are for daily/international spending).
- **Spreadsheet-Style Ledger**:
  - Columns: `Date` | `Description` | `Category` | `Money In (PKR)` | `Money Out (PKR)` | `Balance (PKR)` | `Actions`.
  - **Editable Opening Balance**: Anchor first row from which the running balance is automatically calculated.
  - **Income / Expense Flow**: Segmented toggle on transaction entry with pre-populated categories ("Income", "Salary", "Freelance", "Pocket Money", "Money Received" for income; "Food", "Subscription", "Transport", "Shopping", "Other", etc. for expense).
  - Custom user-defined categories.
  - Filter by date range (All, This Month, Last 30 Days, Custom) and category, plus live search.

### 2. Subscriptions Tracker

- Dedicated recurring subscriptions manager.
- Pre-seeded on first launch with:
  - **HBO MAX** — Yearly — Rs 8,200 (Started Sep 2026)
  - **Spotify** — Monthly — Rs 400
- **Automated Due-Date Deduction**: On due dates (including catching up on any cycles missed while the app was closed), an automatic "Money Out" transaction is generated in the linked card's ledger with category "Subscription" and the `[Auto]` tag.
- **Prorated Cost Summary**: Yearly and custom intervals are prorated to a monthly figure and factored into savings plan calculations.

### 3. Custom Savings Plans & Feasibility Engine

- Create savings plans with Target Amount (PKR), Deadline Date, and Monthly Income (PKR).
- **Auto-Calculated Metrics**:
  - Months remaining until deadline.
  - Required savings per month (`target_amount / months_remaining`).
  - Prorated monthly subscriptions cost.
  - Recommended monthly spending limit (`income - required_savings - subscriptions`).
  - **Feasibility Check**: Warns immediately if required savings exceed monthly income, displaying exact shortage.
  - Optional user override for monthly spending limit.
- **Real Spending Integration**: Sums actual "Money Out" from attached card(s) per calendar month.
- **Surplus & Overspending**:
  - Under-spending produces surplus added to running savings.
  - Overspending is clearly displayed in red with exact overspend amounts, with an optional toggle to deduct overspending from the running surplus.
  - Per-month expandable breakdown table.
- **Deadline Expiration & Windows Toast Notifications**:
  - When a plan's deadline date passes, triggers a native Windows Toast notification (`windows::UI::Notifications`).
  - Moves the plan to "Savings Plan History" displaying final savings results and achievement status (`🎉 Goal Achieved` or `⚠️ Goal Missed`).

### 4. Robust Local Persistence & System Clock

- **Zero-Configuration Autosave**: All data is automatically saved on every change to `%APPDATA%\SavingsTracker\data.json`.
- **Atomic File Writes**: Writes to a `.tmp` file and atomically renames to prevent any file corruption even during unexpected shutdowns.
- **System Clock Integration**: Uses `chrono::Local::now()` for all dates and automated catchup.

---

## 🚀 Building & Running

### Prerequisites

- Windows 10 or 11
- Rust toolchain (`cargo` and `rustc`)

### Running in Development Mode

```powershell
cargo run
```

### Compiling Distributable Release Binary

To build the single, self-contained standalone `.exe`:

```powershell
cargo build --release
```

The resulting standalone executable will be located at:

```
D:\Savings Tracker App\target\release\savings_tracker.exe
```

- **No external runtime dependencies**: The binary contains the entire UI engine, rendering pipeline, and data layer.
- **No console popup**: Configured with `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` so it runs as a pure native Windows window.

---

## 🧪 Running Unit Tests

All business logic is covered by automated unit tests:

```powershell
cargo test
```

Verifies:

- PKR currency formatting with thousands separators
- Chronological running balance calculation from opening balance
- Subscription cycle advancement and prorated calculations
- Automated missed due date catchup and transaction generation
- Savings plan metrics, feasibility detection, and surplus tracking
- Atomic disk save and recovery
