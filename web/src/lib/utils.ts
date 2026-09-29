import type { MonthBreakdown, PlanCalculation, SavingsPlan, Subscription, Transaction } from "../types/index.ts";

/**
 * Parses user-entered PKR currency text by stripping commas and whitespace.
 * Replicates the desktop app's `parse_pkr_input`.
 */
export function parsePkrInput(raw: string): number | null {
  if (!raw) return null;
  const cleaned = raw.replace(/,/g, "").trim();
  if (cleaned === "" || isNaN(Number(cleaned))) {
    return null;
  }
  const val = parseFloat(cleaned);
  return isFinite(val) ? val : null;
}

/**
 * Formats a float as PKR currency with commas and 2 decimals.
 * Example: 1234567.89 -> "Rs 1,234,567.89"
 */
export function formatPkr(amount: number): string {
  const isNeg = amount < 0;
  const absVal = Math.abs(amount);
  const parts = absVal.toFixed(2).split(".");
  const intPart = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  const fracPart = parts[1];
  return `${isNeg ? "-Rs " : "Rs "}${intPart}.${fracPart}`;
}

/**
 * Formats a whole PKR amount without decimals.
 * Example: 106736 -> "Rs 106,736"
 */
export function formatPkrWhole(amount: number): string {
  const isNeg = amount < 0;
  const absVal = Math.round(Math.abs(amount));
  const intStr = absVal.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  return `${isNeg ? "-Rs " : "Rs "}${intStr}`;
}

/**
 * Formats an ISO date string (YYYY-MM-DD) as "29 Sep 2026".
 */
export function formatDate(dateStr: string): string {
  if (!dateStr) return "";
  const parts = dateStr.split("-");
  if (parts.length !== 3) return dateStr;
  const y = parseInt(parts[0], 10);
  const m = parseInt(parts[1], 10) - 1;
  const d = parseInt(parts[2], 10);

  const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
  const mName = months[m] || "";
  return `${d} ${mName} ${y}`;
}

/**
 * Returns today's date formatted as YYYY-MM-DD.
 */
export function todayIso(): string {
  const d = new Date();
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

/**
 * UUID v4 generator using browser crypto
 */
export function generateUuid(): string {
  if (typeof crypto !== "undefined" && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === "x" ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

/**
 * Advances a subscription date by its billing cycle ('monthly', 'yearly', 'custom:N')
 */
export function advanceSubscriptionDate(dateStr: string, cycleStr: string): string {
  const [y, m, d] = dateStr.split("-").map((x) => parseInt(x, 10));
  const date = new Date(y, m - 1, d);

  const normalized = cycleStr.toLowerCase().trim();
  if (normalized === "monthly") {
    date.setMonth(date.getMonth() + 1);
  } else if (normalized === "yearly") {
    date.setFullYear(date.getFullYear() + 1);
  } else if (normalized.startsWith("custom:")) {
    const days = parseInt(normalized.split(":")[1], 10) || 30;
    date.setDate(date.getDate() + days);
  } else {
    date.setMonth(date.getMonth() + 1);
  }

  const resY = date.getFullYear();
  const resM = String(date.getMonth() + 1).padStart(2, "0");
  const resD = String(date.getDate()).padStart(2, "0");
  return `${resY}-${resM}-${resD}`;
}

/**
 * Calculates prorated monthly cost of a single subscription.
 */
export function calcSubscriptionMonthlyCost(sub: Subscription): number {
  if (sub.is_paused || sub.deleted_at) {
    return 0;
  }
  const cycle = (sub.billing_cycle || "monthly").toLowerCase().trim();
  if (cycle === "monthly") {
    return sub.amount;
  }
  if (cycle === "yearly") {
    return sub.amount / 12.0;
  }
  if (cycle.startsWith("custom:")) {
    const days = parseInt(cycle.split(":")[1], 10) || 30;
    if (days <= 0) return 0;
    return sub.amount * (30.4375 / days);
  }
  return sub.amount;
}

/**
 * Calculates total monthly subscription cost across all or specified cards.
 */
export function totalMonthlySubscriptionCost(
  subscriptions: Subscription[],
  cardFilter?: string[]
): number {
  return subscriptions
    .filter((sub) => {
      if (sub.is_paused || sub.deleted_at) return false;
      if (cardFilter && cardFilter.length > 0) {
        return cardFilter.includes(sub.card_id);
      }
      return true;
    })
    .reduce((sum, sub) => sum + calcSubscriptionMonthlyCost(sub), 0);
}

/**
 * Calculates months remaining between today and deadline.
 */
export function calculateMonthsRemaining(today: string, deadline: string): number {
  if (deadline <= today) return 0;
  const d1 = new Date(today);
  const d2 = new Date(deadline);
  const diffMs = d2.getTime() - d1.getTime();
  const diffDays = diffMs / (1000 * 60 * 60 * 24);
  const months = diffDays / 30.4375;
  return months < 1.0 ? 1.0 : months;
}

/**
 * Evaluates savings plan metrics (required savings, recommended limit, feasibility).
 */
export function evaluatePlanMetrics(
  today: string,
  targetAmount: number,
  deadline: string,
  monthlyIncome: number,
  spendingLimitOverride?: number | null,
  proratedSubCost: number = 0
) {
  const monthsRemaining = calculateMonthsRemaining(today, deadline);
  const requiredSavings = monthsRemaining > 0 ? targetAmount / monthsRemaining : targetAmount;
  const recommendedLimit = Math.max(0, monthlyIncome - requiredSavings - proratedSubCost);
  const effectiveLimit = spendingLimitOverride ?? recommendedLimit;

  const isFeasible =
    requiredSavings <= monthlyIncome && monthlyIncome - requiredSavings - proratedSubCost >= 0;

  let shortage = 0;
  if (requiredSavings > monthlyIncome) {
    shortage = requiredSavings - monthlyIncome;
  } else if (monthlyIncome - requiredSavings < proratedSubCost) {
    shortage = proratedSubCost - (monthlyIncome - requiredSavings);
  }

  return {
    monthsRemaining,
    requiredSavings,
    recommendedLimit,
    effectiveLimit,
    isFeasible,
    shortage,
  };
}

/**
 * Computes live plan calculation and monthly breakdowns mirroring SavingsEngine in src/savings.rs
 */
export function computePlanProgress(
  plan: SavingsPlan,
  allTransactions: Transaction[],
  proratedSubCost: number,
  today: string,
  deductOverspending: boolean
): PlanCalculation {
  const dToday = new Date(today);
  const dDeadline = new Date(plan.deadline);
  const diffMs = dDeadline.getTime() - dToday.getTime();
  const daysRemaining = Math.ceil(diffMs / (1000 * 60 * 60 * 24));
  const isExpired = daysRemaining <= 0 || !plan.is_active;

  const {
    monthsRemaining,
    requiredSavings,
    recommendedLimit,
    effectiveLimit,
    isFeasible,
    shortage,
  } = evaluatePlanMetrics(
    today,
    plan.target_amount,
    plan.deadline,
    plan.monthly_income,
    plan.spending_limit,
    proratedSubCost
  );

  // Filter relevant money-out transactions on linked cards from plan.created_at up to today
  const linkedCardIds = plan.linked_card_ids || [];
  const linkedTxs = allTransactions.filter((tx) => {
    if (tx.deleted_at || tx.is_income) return false;
    if (linkedCardIds.length > 0 && !linkedCardIds.includes(tx.card_id)) return false;
    return tx.date >= plan.created_at && tx.date <= today;
  });

  const breakdowns: MonthBreakdown[] = [];
  const createdDate = new Date(plan.created_at);
  let currY = createdDate.getFullYear();
  let currM = createdDate.getMonth() + 1; // 1-12

  const endY = dToday.getFullYear();
  const endM = dToday.getMonth() + 1;

  const monthNames = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"
  ];

  while (currY < endY || (currY === endY && currM <= endM)) {
    const ymPrefix = `${currY}-${String(currM).padStart(2, "0")}`;
    const monthSpent = linkedTxs
      .filter((tx) => tx.date.startsWith(ymPrefix))
      .reduce((sum, tx) => sum + tx.amount, 0);

    const surplus = effectiveLimit - monthSpent;
    const isOverspent = monthSpent > effectiveLimit;
    const isCurrent = currY === endY && currM === endM;

    breakdowns.push({
      year: currY,
      month: currM,
      label: `${monthNames[currM - 1]} ${currY}`,
      monthly_income: plan.monthly_income,
      spending_limit: effectiveLimit,
      actual_spent: monthSpent,
      surplus,
      is_overspent: isOverspent,
      is_current_month: isCurrent,
    });

    if (currM === 12) {
      currY += 1;
      currM = 1;
    } else {
      currM += 1;
    }
  }

  const grossSurplus = breakdowns.reduce((sum, b) => sum + (b.surplus > 0 ? b.surplus : 0), 0);
  const netSurplus = Math.max(0, breakdowns.reduce((sum, b) => sum + b.surplus, 0));
  const totalSaved = deductOverspending ? netSurplus : grossSurplus;
  const percentReached =
    plan.target_amount > 0
      ? Math.min(500, Math.max(0, (totalSaved / plan.target_amount) * 100))
      : 100;

  return {
    months_remaining: monthsRemaining,
    required_savings_per_month: requiredSavings,
    prorated_subscription_cost: proratedSubCost,
    recommended_limit: recommendedLimit,
    effective_limit: effectiveLimit,
    is_feasible: isFeasible,
    shortage,
    total_saved: totalSaved,
    gross_surplus: grossSurplus,
    percent_reached: percentReached,
    days_remaining: daysRemaining,
    is_expired: isExpired,
    breakdowns,
  };
}
