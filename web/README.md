# Savings Tracker — iPhone Access via Web App (PWA)

This is **Part 2** of the Savings Tracker system. It provides an installable Progressive Web App (PWA) designed specifically for iPhone Safari ("Add to Home Screen"), syncing with the same Supabase Postgres database as your Windows desktop app.

---

## Features & iOS Adaptation

- **Thumb-Reachable Bottom Tab Bar**: Standard iOS 3-tab navigation (`Bank Accounts`, `Subscriptions`, `Savings Plans`) replacing desktop sidebar.
- **Stacked Transaction Cards**: Mobile-first layout with tap-to-edit modal, running balance, and color-coded inflow/outflow amounts.
- **Edge-to-Edge iPhone Layout**: Respects Safe Area Insets (`env(safe-area-inset-top)`, `env(safe-area-inset-bottom)`), notch/Dynamic Island, and home indicator.
- **Design Parity**: Uses exact same design tokens (`--bg: #11141A`, `--panel: #171B23`, `--accent: #6C7CF0`, `--in: #3FCDA8`, `--out: #F2726B`) and fonts (`Space Grotesk`, `Inter`, `IBM Plex Mono`).
- **Offline Resilience via IndexedDB & Service Worker**:
  - App shell cached using stale-while-revalidate strategy (`public/sw.js`).
  - IndexedDB storage for offline transactions, subscriptions, and savings plans.
  - Offline mutation queue automatically pushes changes when internet reconnects.
  - Same Last-Write-Wins conflict resolution as desktop.
- **iPhone Web Push Notifications**:
  - Supports iOS Safari Web Push for savings plan deadlines.
  - Stores push subscriptions in `push_subscriptions` table in Supabase.
  - Supabase Edge Function (`supabase/functions/check-deadlines`) checks deadlines and sends push notifications.
- **Comma-Tolerant Currency Input**:
  - Full parity with desktop `parse_pkr_input` — ignores commas in user input.

---

## 1. Running Locally for Development

```bash
cd web
npm install
npm run dev
```

Open `http://localhost:3000` in your browser (or open via your local network IP on your iPhone).

---

## 2. Running Unit Tests

```bash
cd web
npm test
```

Runs the 10 parity tests covering currency parsing, math calculations, subscription proration, savings engine breakdowns, and sync conflict resolution.

---

## 3. Building for Production

```bash
cd web
npm run build
```

This compiles a 100% static single-page app inside `web/dist/`.

---

## 4. Deploying to Free Static Hosting

Because this is a pure client-side PWA talking directly to Supabase via `@supabase/supabase-js`, it can be hosted on any static hosting provider at no cost:

### Option A: Cloudflare Pages
1. Push your repository to GitHub / GitLab.
2. In Cloudflare Dashboard -> **Workers & Pages** -> **Create application** -> **Pages**.
3. Select your repository.
4. Set Build Settings:
   - Root directory: `web`
   - Build command: `npm run build`
   - Build output directory: `dist`
5. (Optional) Set Environment Variables:
   - `VITE_SUPABASE_URL`: `https://<your-project>.supabase.co`
   - `VITE_SUPABASE_ANON_KEY`: `<your-anon-key>`
   - `VITE_VAPID_PUBLIC_KEY`: `<your-vapid-public-key>`

### Option B: Vercel or Netlify
- Build command: `cd web && npm run build`
- Publish directory: `web/dist`

---

## 5. Installing on iPhone

1. Open your deployed URL (e.g. `https://your-savings-app.pages.dev`) in **Safari** on your iPhone.
2. Sign in with the exact same email/password as your Windows desktop app.
3. Tap Safari's **Share** button (the square with an arrow pointing up at the bottom toolbar).
4. Scroll down and tap **"Add to Home Screen"**.
5. Tap **Add** in the top-right corner.
6. The Savings Tracker icon will appear on your iPhone Home Screen. When opened, it runs in standalone mode without any browser chrome or URL bars!
