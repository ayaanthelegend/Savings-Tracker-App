# Supabase Edge Function: `check-deadlines`

This function mirrors the desktop app's `check_expirations` logic (from [`src/savings.rs`](../../src/savings.rs)). It scans for active savings plans whose deadline has arrived or passed, calculates final savings progress, marks the plan closed/inactive, and pushes a Web Push notification to the user's iPhone PWA.

---

## 1. Prerequisites & Secrets

In your Supabase Dashboard -> **Edge Functions** -> **Secrets** (or via Supabase CLI), configure:
- `SUPABASE_URL`: Your Supabase project URL (automatically set on hosted Supabase)
- `SUPABASE_SERVICE_ROLE_KEY`: Your project's service role key (automatically set on hosted Supabase)
- `VAPID_PUBLIC_KEY`: Your VAPID public key
- `VAPID_PRIVATE_KEY`: Your VAPID private key
- `VAPID_SUBJECT`: `mailto:your-email@example.com`

*(You can generate a VAPID keypair using `npx web-push generate-vapid-keys` or an online generator).*

---

## 2. Deploying the Function

Using the Supabase CLI:

```bash
supabase functions deploy check-deadlines --no-verify-jwt
```

---

## 3. Scheduling via Supabase Cron (pg_cron)

You can trigger this function daily at 09:00 UTC using PostgreSQL `pg_cron` and `pg_net` in the Supabase SQL Editor:

```sql
select cron.schedule(
    'check-savings-deadlines-daily',
    '0 9 * * *',
    $$
    select net.http_post(
        url := 'https://<your-project-ref>.supabase.co/functions/v1/check-deadlines',
        headers := '{"Content-Type": "application/json", "Authorization": "Bearer <service-role-key>"}'::jsonb,
        body := '{}'::jsonb
    ) as request_id;
    $$
);
```

---

## 4. Manual Testing

You can trigger a check at any time with curl:

```bash
curl -i --location --request POST 'https://<your-project-ref>.supabase.co/functions/v1/check-deadlines' \
  --header 'Authorization: Bearer <service-role-key>' \
  --header 'Content-Type: application/json'
```
