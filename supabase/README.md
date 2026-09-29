# Supabase Cloud Sync Setup Guide for Savings Tracker

This directory contains the database schema and security policies required for Cloud Sync between your Windows desktop app and the iPhone PWA (Part 2).

---

## 1. Quick Setup in Supabase Dashboard

1. **Create Project**: Go to [https://supabase.com](https://supabase.com) and create a new project.
2. **Execute Schema**:
   - Open the **SQL Editor** from the left navigation.
   - Click **New Query**.
   - Copy the entire contents of [`supabase/schema.sql`](./schema.sql) and paste it into the query editor.
   - Click **Run** (or `Ctrl+Enter`).
   - This creates all 5 tables (`cards`, `transactions`, `subscriptions`, `savings_plans`, `categories`), automatic `updated_at` triggers, sync indexes, and Row Level Security (RLS) policies.
3. **Get API Credentials**:
   - Go to **Project Settings** -> **API**.
   - Copy:
     - **Project URL** (e.g. `https://your-project.supabase.co`)
     - **anon public key** (starts with `eyJ...`)
4. **Auth Settings (Email/Password)**:
   - Go to **Authentication** -> **Providers** -> **Email**.
   - Ensure Email provider is enabled.
   - *(Optional for frictionless single-user access)*: Under **Authentication** -> **Email Auth**, you can toggle OFF "Confirm email" if you'd like your account ready immediately upon registration without email verification.

---

## 2. Desktop App Authentication

Launch the desktop app:
- On first launch, the app displays the **Cloud Sync Login** screen.
- Click **Configure Supabase Project** and enter:
  - Supabase Project URL
  - Supabase Anon Key
- Enter your email and password.
- Click **Create New Account** (if first time) or **Log In**.
- Once authenticated:
  - The session is saved locally in `%APPDATA%\SavingsTracker\session.json`.
  - Next time you open the app, it silently logs you in using the stored refresh token.
  - The sync worker automatically pulls updates every 60s while the window is focused (or immediately upon focus) and pushes any local changes.
  - To log out, click **Log out** at the bottom of the left sidebar.

---

## 3. Environment Variables (Optional Alternative)

If you prefer configuring credentials via environment variables instead of the login screen UI, set:
- `SUPABASE_URL`: e.g. `https://xyz.supabase.co`
- `SUPABASE_ANON_KEY`: your anon public key

---

## 4. Tables and Schema Reference

| Table | Description | Sync Strategy |
|---|---|---|
| `cards` | Bank accounts & cards | UUID primary key, RLS `user_id = auth.uid()`, soft-delete via `deleted_at` |
| `transactions` | Ledger transactions | Linked to `cards(id)`, filtered by `deleted_at IS NULL` |
| `subscriptions` | Recurring subscriptions | Billing cycle serialized as `monthly`, `yearly`, or `custom:N` |
| `savings_plans` | Savings goals & monthly limits | Config persisted; progress recomputed client-side |
| `categories` | Custom transaction categories | Defaults: Food, Income, Subscription, Transport, Shopping, Other |

### Conflict Resolution
- **Last-Write-Wins** by `updated_at`.
- On exact timestamp tie, the server record deterministically wins.
- Soft-deletes bump `updated_at` to the deletion timestamp, ensuring deletions reliably propagate and beat older stale edits.
