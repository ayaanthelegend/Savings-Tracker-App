-- Migration: Add push_subscriptions table for iPhone PWA Web Push
create table if not exists public.push_subscriptions (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade not null,
    endpoint text not null unique,
    p256dh_key text not null,
    auth_key text not null,
    created_at timestamptz not null default now()
);

create index if not exists idx_push_subscriptions_user on public.push_subscriptions(user_id);

alter table public.push_subscriptions enable row level security;

drop policy if exists "Push subscriptions user isolation" on public.push_subscriptions;
create policy "Push subscriptions user isolation" on public.push_subscriptions
    for all
    using (auth.uid() = user_id)
    with check (auth.uid() = user_id);
