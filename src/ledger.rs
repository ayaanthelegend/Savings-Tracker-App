use crate::models::{Card, Transaction};
use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LedgerRow {
    pub transaction_id: Option<Uuid>,
    pub date: NaiveDate,
    pub description: String,
    pub category: String,
    pub money_in: Option<f64>,
    pub money_out: Option<f64>,
    pub running_balance: f64,
    pub is_opening_balance: bool,
    pub auto_generated: bool,
}

pub struct LedgerCalculator;

impl LedgerCalculator {
    /// Computes full chronological ledger rows for a card starting from its opening balance.
    /// Each row contains its exact running balance.
    pub fn compute_full_ledger(card: &Card, all_transactions: &[Transaction]) -> Vec<LedgerRow> {
        let mut indexed_txs: Vec<(usize, &Transaction)> = all_transactions
            .iter()
            .enumerate()
            .filter(|(_, t)| t.card_id == card.id && t.deleted_at.is_none())
            .collect();

        // Sort chronologically ascending: by date, then preserve relative position in all_transactions
        indexed_txs.sort_by(|(idx_a, a), (idx_b, b)| a.date.cmp(&b.date).then_with(|| idx_a.cmp(idx_b)));

        let card_txs: Vec<&Transaction> = indexed_txs.into_iter().map(|(_, t)| t).collect();

        let mut rows = Vec::with_capacity(card_txs.len() + 1);

        // Opening balance row
        let first_date = card.opening_balance_date.unwrap_or_else(|| {
            card_txs.first().map(|t| t.date).unwrap_or_else(|| chrono::Local::now().date_naive())
        });
        let mut current_balance = card.opening_balance;

        rows.push(LedgerRow {
            transaction_id: None,
            date: first_date,
            description: card.opening_balance_description.clone(),
            category: "Initial Balance".to_string(),
            money_in: if card.opening_balance > 0.0 { Some(card.opening_balance) } else { None },
            money_out: if card.opening_balance < 0.0 { Some(card.opening_balance.abs()) } else { None },
            running_balance: current_balance,
            is_opening_balance: true,
            auto_generated: false,
        });

        for tx in card_txs {
            if tx.is_income {
                current_balance += tx.amount;
                rows.push(LedgerRow {
                    transaction_id: Some(tx.id),
                    date: tx.date,
                    description: tx.description.clone(),
                    category: tx.category.clone(),
                    money_in: Some(tx.amount),
                    money_out: None,
                    running_balance: current_balance,
                    is_opening_balance: false,
                    auto_generated: tx.auto_generated,
                });
            } else {
                current_balance -= tx.amount;
                rows.push(LedgerRow {
                    transaction_id: Some(tx.id),
                    date: tx.date,
                    description: tx.description.clone(),
                    category: tx.category.clone(),
                    money_in: None,
                    money_out: Some(tx.amount),
                    running_balance: current_balance,
                    is_opening_balance: false,
                    auto_generated: tx.auto_generated,
                });
            }
        }

        rows
    }

    /// Filters and sorts ledger rows according to current UI filters.
    pub fn filter_and_sort_ledger(
        full_rows: &[LedgerRow],
        category_filter: Option<&str>,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
        search_query: &str,
        sort_descending: bool,
    ) -> Vec<LedgerRow> {
        let q = search_query.trim().to_lowercase();

        let mut filtered: Vec<LedgerRow> = full_rows
            .iter()
            .filter(|row| {
                if let Some(from) = date_from {
                    if row.date < from && !row.is_opening_balance {
                        return false;
                    }
                }
                if let Some(to) = date_to {
                    if row.date > to && !row.is_opening_balance {
                        return false;
                    }
                }

                if let Some(cat) = category_filter {
                    if !cat.is_empty() && cat != "All" && !row.is_opening_balance {
                        if !row.category.eq_ignore_ascii_case(cat) {
                            return false;
                        }
                    }
                }

                if !q.is_empty() {
                    let desc_match = row.description.to_lowercase().contains(&q);
                    let cat_match = row.category.to_lowercase().contains(&q);
                    if !desc_match && !cat_match {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        if sort_descending {
            filtered.reverse();
        }

        filtered
    }
}
