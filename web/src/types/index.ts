export type TableName = "cards" | "transactions" | "subscriptions" | "savings_plans" | "categories";

export interface Card {
  id: string;
  user_id?: string;
  name: string;
  is_primary: boolean;
  opening_balance: number;
  opening_balance_description: string;
  opening_balance_date?: string | null;
  updated_at: string;
  deleted_at?: string | null;
}

export interface Transaction {
  id: string;
  user_id?: string;
  card_id: string;
  date: string; // YYYY-MM-DD
  description: string;
  category: string;
  amount: number;
  is_income: boolean;
  auto_generated: boolean;
  updated_at: string;
  deleted_at?: string | null;
}

export interface Subscription {
  id: string;
  user_id?: string;
  card_id: string;
  name: string;
  amount: number;
  billing_cycle: string; // 'monthly' | 'yearly' | 'custom:N'
  start_date?: string | null;
  next_due_date: string; // YYYY-MM-DD
  is_paused: boolean;
  updated_at: string;
  deleted_at?: string | null;
}

export interface SavingsPlan {
  id: string;
  user_id?: string;
  name: string;
  target_amount: number;
  deadline: string; // YYYY-MM-DD
  monthly_income: number;
  spending_limit?: number | null;
  is_active: boolean;
  deduct_overspending: boolean;
  linked_card_ids: string[];
  created_at: string; // YYYY-MM-DD
  updated_at: string;
  deleted_at?: string | null;
}

export interface Category {
  id: string;
  user_id?: string;
  name: string;
  is_default: boolean;
  updated_at: string;
  deleted_at?: string | null;
}

export interface PushSubscriptionRecord {
  id?: string;
  user_id: string;
  endpoint: string;
  p256dh_key: string;
  auth_key: string;
  created_at?: string;
}

export interface SyncWatermarks {
  cards?: string;
  transactions?: string;
  subscriptions?: string;
  savings_plans?: string;
  categories?: string;
}

export interface SyncQueueItem {
  id: string;
  table: TableName;
  record: any;
  timestamp: number;
}

export interface MonthBreakdown {
  year: number;
  month: number;
  label: string;
  monthly_income: number;
  spending_limit: number;
  actual_spent: number;
  surplus: number;
  is_overspent: boolean;
  is_current_month: boolean;
}

export interface PlanCalculation {
  months_remaining: number;
  required_savings_per_month: number;
  prorated_subscription_cost: number;
  recommended_limit: number;
  effective_limit: number;
  is_feasible: boolean;
  shortage: number;
  total_saved: number;
  gross_surplus: number;
  percent_reached: number;
  days_remaining: number;
  is_expired: boolean;
  breakdowns: MonthBreakdown[];
}
