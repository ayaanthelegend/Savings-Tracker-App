use crate::models::{AppData, SavingsPlan, Transaction};
use crate::notification::show_toast_notification;
use crate::subscriptions::SubscriptionManager;
use chrono::{Datelike, NaiveDate};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MonthBreakdown {
    pub year: i32,
    pub month: u32,
    pub label: String,
    pub monthly_income: f64,
    pub spending_limit: f64,
    pub actual_spent: f64,
    pub surplus: f64,
    pub is_overspent: bool,
    pub is_current_month: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlanCalculation {
    pub months_remaining: f64,
    pub required_savings_per_month: f64,
    pub prorated_subscription_cost: f64,
    pub recommended_limit: f64,
    pub effective_limit: f64,
    pub is_feasible: bool,
    pub shortage: f64,
    pub total_saved: f64,
    pub gross_surplus: f64,
    pub percent_reached: f64,
    pub days_remaining: i64,
    pub is_expired: bool,
    pub breakdowns: Vec<MonthBreakdown>,
}

pub struct SavingsEngine;

impl SavingsEngine {
    /// Calculates months remaining between today and deadline.
    /// Returns at least 1.0 month if within the current month to avoid division by zero.
    pub fn calculate_months_remaining(today: NaiveDate, deadline: NaiveDate) -> f64 {
        if deadline <= today {
            return 0.0;
        }

        let total_days = (deadline - today).num_days() as f64;
        let months = total_days / 30.4375;
        if months < 1.0 {
            1.0
        } else {
            months
        }
    }

    /// Evaluates recommended parameters for a proposed or existing plan.
    pub fn evaluate_plan_metrics(
        today: NaiveDate,
        target_amount: f64,
        deadline: NaiveDate,
        monthly_income: f64,
        spending_limit_override: Option<f64>,
        prorated_subscription_cost: f64,
    ) -> (f64, f64, f64, f64, bool, f64) {
        let months_remaining = Self::calculate_months_remaining(today, deadline);
        let required_savings = if months_remaining > 0.0 {
            target_amount / months_remaining
        } else {
            target_amount
        };

        let recommended_limit = (monthly_income - required_savings - prorated_subscription_cost).max(0.0);
        let effective_limit = spending_limit_override.unwrap_or(recommended_limit);

        let is_feasible = required_savings <= monthly_income && (monthly_income - required_savings - prorated_subscription_cost) >= 0.0;
        let shortage = if required_savings > monthly_income {
            required_savings - monthly_income
        } else if monthly_income - required_savings < prorated_subscription_cost {
            prorated_subscription_cost - (monthly_income - required_savings)
        } else {
            0.0
        };

        (
            months_remaining,
            required_savings,
            recommended_limit,
            effective_limit,
            is_feasible,
            shortage,
        )
    }

    /// Computes full live calculation and historical monthly breakdowns for a plan.
    pub fn compute_progress(
        plan: &SavingsPlan,
        all_transactions: &[Transaction],
        prorated_sub_cost: f64,
        today: NaiveDate,
        deduct_overspending: bool,
    ) -> PlanCalculation {
        // If already closed, use final recorded amounts if available
        if plan.closed {
            let final_val = plan.final_saved.unwrap_or(0.0);
            let pct = if plan.target_amount > 0.0 {
                (final_val / plan.target_amount * 100.0).clamp(0.0, 500.0)
            } else {
                100.0
            };

            return PlanCalculation {
                months_remaining: 0.0,
                required_savings_per_month: 0.0,
                prorated_subscription_cost: prorated_sub_cost,
                recommended_limit: plan.spending_limit_override.unwrap_or(0.0),
                effective_limit: plan.spending_limit_override.unwrap_or(0.0),
                is_feasible: true,
                shortage: 0.0,
                total_saved: final_val,
                gross_surplus: final_val,
                percent_reached: pct,
                days_remaining: 0,
                is_expired: true,
                breakdowns: Vec::new(),
            };
        }

        let days_remaining = (plan.deadline - today).num_days();
        let is_expired = days_remaining <= 0;

        let (months_rem, req_savings, rec_limit, eff_limit, feasible, shortage) =
            Self::evaluate_plan_metrics(
                today,
                plan.target_amount,
                plan.deadline,
                plan.monthly_income,
                plan.spending_limit_override,
                prorated_sub_cost,
            );

        // Filter transactions for linked cards
        let linked_txs: Vec<&Transaction> = all_transactions
            .iter()
            .filter(|t| {
                if t.deleted_at.is_some() {
                    return false;
                }
                if !plan.linked_card_ids.is_empty() && !plan.linked_card_ids.contains(&t.card_id) {
                    return false;
                }
                // Transactions from plan creation month up to today
                t.date >= plan.created_at && t.date <= today && !t.is_income
            })
            .collect();

        // Generate list of months from plan.created_at to today
        let mut breakdowns = Vec::new();
        let mut curr_y = plan.created_at.year();
        let mut curr_m = plan.created_at.month();

        let end_y = today.year();
        let end_m = today.month();

        while curr_y < end_y || (curr_y == end_y && curr_m <= end_m) {
            // Sum spending for this year & month
            let month_spent: f64 = linked_txs
                .iter()
                .filter(|t| t.date.year() == curr_y && t.date.month() == curr_m)
                .map(|t| t.amount)
                .sum();

            let surplus = eff_limit - month_spent;
            let is_overspent = month_spent > eff_limit;
            let is_curr = curr_y == end_y && curr_m == end_m;

            let month_name = match curr_m {
                1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr", 5 => "May", 6 => "Jun",
                7 => "Jul", 8 => "Aug", 9 => "Sep", 10 => "Oct", 11 => "Nov", _ => "Dec",
            };

            breakdowns.push(MonthBreakdown {
                year: curr_y,
                month: curr_m,
                label: format!("{} {}", month_name, curr_y),
                monthly_income: plan.monthly_income,
                spending_limit: eff_limit,
                actual_spent: month_spent,
                surplus,
                is_overspent,
                is_current_month: is_curr,
            });

            if curr_m == 12 {
                curr_y += 1;
                curr_m = 1;
            } else {
                curr_m += 1;
            }
        }

        // Calculate total saved:
        let gross_surplus: f64 = breakdowns
            .iter()
            .map(|b| if b.surplus > 0.0 { b.surplus } else { 0.0 })
            .sum();

        let net_surplus: f64 = breakdowns.iter().map(|b| b.surplus).sum::<f64>().max(0.0);

        let total_saved = if deduct_overspending {
            net_surplus
        } else {
            gross_surplus
        };

        let percent_reached = if plan.target_amount > 0.0 {
            (total_saved / plan.target_amount * 100.0).clamp(0.0, 500.0)
        } else {
            100.0
        };

        PlanCalculation {
            months_remaining: months_rem,
            required_savings_per_month: req_savings,
            prorated_subscription_cost: prorated_sub_cost,
            recommended_limit: rec_limit,
            effective_limit: eff_limit,
            is_feasible: feasible,
            shortage,
            total_saved,
            gross_surplus,
            percent_reached,
            days_remaining,
            is_expired,
            breakdowns,
        }
    }

    /// Checks all active plans against system date `today`.
    pub fn check_expirations(data: &mut AppData, today: NaiveDate) {
        let now = chrono::Utc::now();
        for plan in &mut data.plans {
            if !plan.closed && plan.deleted_at.is_none() && today >= plan.deadline {
                let prorated_sub = SubscriptionManager::total_monthly_cost(
                    &data.subscriptions,
                    if plan.linked_card_ids.is_empty() {
                        None
                    } else {
                        Some(&plan.linked_card_ids)
                    },
                );

                let calc = Self::compute_progress(plan, &data.transactions, prorated_sub, today, true);

                plan.closed = true;
                plan.closed_at = Some(today);
                plan.final_saved = Some(calc.total_saved);
                let hit_goal = calc.total_saved >= plan.target_amount;
                plan.goal_met = Some(hit_goal);
                plan.updated_at = now;

                // Native Windows Toast Notification
                let title = format!("Savings Plan Ended: {}", plan.name);
                let body = if hit_goal {
                    format!(
                        "Congratulations! You reached your goal of Rs {:.0} by saving Rs {:.0}!",
                        plan.target_amount, calc.total_saved
                    )
                } else {
                    format!(
                        "Plan completed: Saved Rs {:.0} of Rs {:.0} target.",
                        calc.total_saved, plan.target_amount
                    )
                };

                show_toast_notification(&title, &body);
            }
        }
    }
}
