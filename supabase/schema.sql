-- Supabase Schema for Savings Tracker Cloud Sync
-- Tables: cards, transactions, subscriptions, savings_plans, categories
-- Includes triggers for automatic updated_at maintenance and Row Level Security (RLS)

-- 1. Helper function for updated_at timestamps
create or replace function public.set_updated_at()
returns trigger as $$
begin
    new.updated_at = now();
    return new;
end;
$$ language plpgsql;

-- 2. Cards table
create table if not exists public.cards (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade not null,
    name text not null,
    is_primary boolean not null default false,
    opening_balance numeric not null default 0,
    opening_balance_description text not null default 'Opening Balance',
    opening_balance_date date,
    updated_at timestamptz not null default now(),
    deleted_at timestamptz
);

-- Index for sync queries
create index if not exists idx_cards_user_updated on public.cards(user_id, updated_at);

-- Trigger for cards updated_at
drop trigger if exists set_cards_updated_at on public.cards;
create trigger set_cards_updated_at
    before update on public.cards
    for each row
    execute function public.set_updated_at();

-- 3. Transactions table
create table if not exists public.transactions (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade not null,
    card_id uuid references public.cards(id) on delete cascade not null,
    date date not null,
    description text not null,
    category text not null,
    amount numeric not null,
    is_income boolean not null,
    auto_generated boolean not null default false,
    updated_at timestamptz not null default now(),
    deleted_at timestamptz
);

-- Index for sync queries
create index if not exists idx_transactions_user_updated on public.transactions(user_id, updated_at);
create index if not exists idx_transactions_card_id on public.transactions(card_id);

-- Trigger for transactions updated_at
drop trigger if exists set_transactions_updated_at on public.transactions;
create trigger set_transactions_updated_at
    before update on public.transactions
    for each row
    execute function public.set_updated_at();

-- 4. Subscriptions table
create table if not exists public.subscriptions (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade not null,
    card_id uuid references public.cards(id) on delete cascade not null,
    name text not null,
    amount numeric not null,
    billing_cycle text not null, -- 'monthly', 'yearly', 'custom:N'
    start_date date,
    next_due_date date not null,
    is_paused boolean not null default false,
    updated_at timestamptz not null default now(),
    deleted_at timestamptz
);

-- Index for sync queries
create index if not exists idx_subscriptions_user_updated on public.subscriptions(user_id, updated_at);
create index if not exists idx_subscriptions_card_id on public.subscriptions(card_id);

-- Trigger for subscriptions updated_at
drop trigger if exists set_subscriptions_updated_at on public.subscriptions;
create trigger set_subscriptions_updated_at
    before update on public.subscriptions
    for each row
    execute function public.set_updated_at();

-- 5. Savings Plans table
-- Note: Derived metrics (progress, surplus, breakdowns) are recomputed client-side from transactions + plan config
create table if not exists public.savings_plans (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade not null,
    name text not null,
    target_amount numeric not null,
    deadline date not null,
    monthly_income numeric not null,
    spending_limit numeric,
    is_active boolean not null default true,
    deduct_overspending boolean not null default false,
    linked_card_ids jsonb default '[]'::jsonb,
    created_at date not null default current_date,
    updated_at timestamptz not null default now(),
    deleted_at timestamptz
);

-- Index for sync queries
create index if not exists idx_savings_plans_user_updated on public.savings_plans(user_id, updated_at);

-- Trigger for savings_plans updated_at
drop trigger if exists set_savings_plans_updated_at on public.savings_plans;
create trigger set_savings_plans_updated_at
    before update on public.savings_plans
    for each row
    execute function public.set_updated_at();

-- 6. Categories table
create table if not exists public.categories (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade not null,
    name text not null,
    is_default boolean not null default false,
    updated_at timestamptz not null default now(),
    deleted_at timestamptz
);

-- Index for sync queries
create index if not exists idx_categories_user_updated on public.categories(user_id, updated_at);

-- Trigger for categories updated_at
drop trigger if exists set_categories_updated_at on public.categories;
create trigger set_categories_updated_at
    before update on public.categories
    for each row
    execute function public.set_updated_at();

-- 7. Push Subscriptions table (for iPhone Web Push PWA)
create table if not exists public.push_subscriptions (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade not null,
    endpoint text not null unique,
    p256dh_key text not null,
    auth_key text not null,
    created_at timestamptz not null default now()
);

-- Index for push subscriptions
create index if not exists idx_push_subscriptions_user on public.push_subscriptions(user_id);

-- 8. Enable Row Level Security (RLS) on all tables
alter table public.cards enable row level security;
alter table public.transactions enable row level security;
alter table public.subscriptions enable row level security;
alter table public.savings_plans enable row level security;
alter table public.categories enable row level security;
alter table public.push_subscriptions enable row level security;

-- 9. RLS Policies: Isolate each user to their own rows
drop policy if exists "Cards user isolation" on public.cards;
create policy "Cards user isolation" on public.cards
    for all
    using (auth.uid() = user_id)
    with check (auth.uid() = user_id);

drop policy if exists "Transactions user isolation" on public.transactions;
create policy "Transactions user isolation" on public.transactions
    for all
    using (auth.uid() = user_id)
    with check (auth.uid() = user_id);

drop policy if exists "Subscriptions user isolation" on public.subscriptions;
create policy "Subscriptions user isolation" on public.subscriptions
    for all
    using (auth.uid() = user_id)
    with check (auth.uid() = user_id);

drop policy if exists "Savings plans user isolation" on public.savings_plans;
create policy "Savings plans user isolation" on public.savings_plans
    for all
    using (auth.uid() = user_id)
    with check (auth.uid() = user_id);

drop policy if exists "Categories user isolation" on public.categories;
create policy "Categories user isolation" on public.categories
    for all
    using (auth.uid() = user_id)
    with check (auth.uid() = user_id);

drop policy if exists "Push subscriptions user isolation" on public.push_subscriptions;
create policy "Push subscriptions user isolation" on public.push_subscriptions
    for all
    using (auth.uid() = user_id)
    with check (auth.uid() = user_id);

