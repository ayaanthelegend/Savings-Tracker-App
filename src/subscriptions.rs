use crate::models::{AppData, BillingCycle, Subscription, Transaction};
use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct YearlyReserveInfo {
    pub monthly_reserve: f64,
    pub total_cost: f64,
    pub months_remaining: u32,
    pub months_accumulated: u32,
    pub saved_amount: f64,
    pub progress_ratio: f32,
    pub due_text: String,
}

pub struct SubscriptionManager;

impl SubscriptionManager {
    /// Advances a due date by one billing cycle.
    pub fn advance_date(date: NaiveDate, cycle: &BillingCycle) -> NaiveDate {
        match cycle {
            BillingCycle::Monthly => date
                .checked_add_months(chrono::Months::new(1))
                .unwrap_or_else(|| date + chrono::Duration::days(30)),
            BillingCycle::Yearly => date
                .checked_add_months(chrono::Months::new(12))
                .unwrap_or_else(|| date + chrono::Duration::days(365)),
            BillingCycle::CustomDays(days) => date + chrono::Duration::days(*days as i64),
        }
    }

    /// Calculates the prorated monthly cost of a single subscription.
    pub fn monthly_cost(sub: &Subscription) -> f64 {
        if sub.paused {
            return 0.0;
        }
        match sub.cycle {
            BillingCycle::Monthly => sub.amount,
            BillingCycle::Yearly => sub.amount / 12.0,
            BillingCycle::CustomDays(days) => {
                if days == 0 {
                    0.0
                } else {
                    sub.amount * (30.4375 / days as f64)
                }
            }
        }
    }

    /// Calculates total monthly subscription cost across all or specified cards.
    pub fn total_monthly_cost(subscriptions: &[Subscription], card_filter: Option<&[Uuid]>) -> f64 {
        subscriptions
            .iter()
            .filter(|sub| {
                if sub.paused || sub.deleted_at.is_some() {
                    return false;
                }
                match card_filter {
                    Some(cards) => cards.is_empty() || cards.contains(&sub.card_id),
                    None => true,
                }
            })
            .map(Self::monthly_cost)
            .sum()
    }

    /// Calculates the monthly sinking fund reserve needed for all active annual subscriptions.
    pub fn annual_reserve_monthly_needed(subscriptions: &[Subscription], card_filter: Option<&[Uuid]>) -> f64 {
        subscriptions
            .iter()
            .filter(|sub| {
                if sub.paused || sub.deleted_at.is_some() || sub.cycle != BillingCycle::Yearly {
                    return false;
                }
                match card_filter {
                    Some(cards) => cards.is_empty() || cards.contains(&sub.card_id),
                    None => true,
                }
            })
            .map(|sub| sub.amount / 12.0)
            .sum()
    }

    /// Evaluates the sinking fund reserve status for a yearly subscription.
    /// Returns None if the subscription is not yearly or is paused/deleted.
    pub fn yearly_reserve_status(sub: &Subscription, today: NaiveDate) -> Option<YearlyReserveInfo> {
        if sub.paused || sub.deleted_at.is_some() || sub.cycle != BillingCycle::Yearly {
            return None;
        }

        let total_cost = sub.amount;
        let monthly_reserve = total_cost / 12.0;

        let days_until = (sub.next_due_date - today).num_days();
        let (months_rem, due_text) = if days_until == 0 {
            (0, "Due today".to_string())
        } else if days_until < 0 {
            (0, "Overdue".to_string())
        } else if days_until <= 7 {
            let m = 1;
            let d_txt = if days_until == 1 {
                "Due tomorrow".to_string()
            } else {
                format!("Due in {} days", days_until)
            };
            (m, d_txt)
        } else {
            let m = ((days_until as f64) / 30.4375).round() as u32;
            let clamped_m = m.clamp(1, 12);
            let d_txt = if clamped_m == 1 {
                "Due in 1 month".to_string()
            } else {
                format!("Due in {} months", clamped_m)
            };
            (clamped_m, d_txt)
        };

        let months_accumulated = 12_u32.saturating_sub(months_rem);
        let saved_amount = (months_accumulated as f64 * monthly_reserve).min(total_cost);
        let progress_ratio = if total_cost > 0.0 {
            (saved_amount / total_cost).clamp(0.0, 1.0) as f32
        } else {
            1.0
        };

        Some(YearlyReserveInfo {
            monthly_reserve,
            total_cost,
            months_remaining: months_rem,
            months_accumulated,
            saved_amount,
            progress_ratio,
            due_text,
        })
    }

    /// Calculates total accumulated savings across all active yearly subscriptions.
    #[allow(dead_code)]
    pub fn total_accumulated_annual_reserve(subscriptions: &[Subscription], today: NaiveDate, card_filter: Option<&[Uuid]>) -> f64 {
        subscriptions
            .iter()
            .filter(|sub| {
                if sub.paused || sub.deleted_at.is_some() || sub.cycle != BillingCycle::Yearly {
                    return false;
                }
                match card_filter {
                    Some(cards) => cards.is_empty() || cards.contains(&sub.card_id),
                    None => true,
                }
            })
            .filter_map(|sub| Self::yearly_reserve_status(sub, today).map(|info| info.saved_amount))
            .sum()
    }

    /// Checks all subscriptions against `today`.
    /// For any subscription where `next_due_date <= today` and not paused,
    /// catches up by creating a matching "Money Out" transaction in the linked card's ledger
    /// and advancing `next_due_date` until it is in the future.
    /// Returns descriptions of generated transactions.
    pub fn process_due_subscriptions(data: &mut AppData, today: NaiveDate) -> Vec<String> {
        let mut generated_logs = Vec::new();
        let now = chrono::Utc::now();

        for sub in &mut data.subscriptions {
            if sub.paused || sub.deleted_at.is_some() {
                continue;
            }

            // Loop to catch up multiple missed intervals if app was closed for a long time
            let mut cycles_missed = 0;
            while sub.next_due_date <= today && cycles_missed < 100 {
                let tx = Transaction {
                    id: Uuid::new_v4(),
                    card_id: sub.card_id,
                    date: sub.next_due_date,
                    description: format!("Subscription: {}", sub.name),
                    category: "Subscription".to_string(),
                    amount: sub.amount,
                    is_income: false,
                    auto_generated: true,
                    updated_at: now,
                    deleted_at: None,
                };
                data.transactions.push(tx);

                generated_logs.push(format!(
                    "Deducted Rs {:.2} for '{}' due on {}",
                    sub.amount, sub.name, sub.next_due_date
                ));

                sub.next_due_date = Self::advance_date(sub.next_due_date, &sub.cycle);
                sub.updated_at = now;
                cycles_missed += 1;
            }
        }

        generated_logs
    }
}
