use crate::models::{AppData, BillingCycle, Subscription, Transaction};
use chrono::NaiveDate;
use uuid::Uuid;

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
                if sub.paused {
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

    /// Checks all subscriptions against `today`.
    /// For any subscription where `next_due_date <= today` and not paused,
    /// catches up by creating a matching "Money Out" transaction in the linked card's ledger
    /// and advancing `next_due_date` until it is in the future.
    /// Returns descriptions of generated transactions.
    pub fn process_due_subscriptions(data: &mut AppData, today: NaiveDate) -> Vec<String> {
        let mut generated_logs = Vec::new();

        for sub in &mut data.subscriptions {
            if sub.paused {
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
                };
                data.transactions.push(tx);

                generated_logs.push(format!(
                    "Deducted Rs {:.2} for '{}' due on {}",
                    sub.amount, sub.name, sub.next_due_date
                ));

                sub.next_due_date = Self::advance_date(sub.next_due_date, &sub.cycle);
                cycles_missed += 1;
            }
        }

        generated_logs
    }
}
