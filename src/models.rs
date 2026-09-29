use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Card {
    pub id: Uuid,
    pub name: String,
    pub is_primary: bool,
    pub opening_balance: f64,
    #[serde(default = "default_opening_balance_description")]
    pub opening_balance_description: String,
    #[serde(default)]
    pub opening_balance_date: Option<NaiveDate>,
}

pub fn default_opening_balance_description() -> String {
    "Opening Balance".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: Uuid,
    pub card_id: Uuid,
    pub date: NaiveDate,
    pub description: String,
    pub category: String,
    pub amount: f64,
    pub is_income: bool,
    #[serde(default)]
    pub auto_generated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BillingCycle {
    Monthly,
    Yearly,
    CustomDays(u32),
}

impl BillingCycle {
    #[allow(dead_code)]
    pub fn display_label(&self) -> String {
        match self {
            BillingCycle::Monthly => "Monthly".to_string(),
            BillingCycle::Yearly => "Yearly".to_string(),
            BillingCycle::CustomDays(days) => format!("Every {} days", days),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subscription {
    pub id: Uuid,
    pub card_id: Uuid,
    pub name: String,
    pub amount: f64,
    pub cycle: BillingCycle,
    pub start_date: NaiveDate,
    pub next_due_date: NaiveDate,
    #[serde(default)]
    pub paused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavingsPlan {
    pub id: Uuid,
    pub name: String,
    pub target_amount: f64,
    pub deadline: NaiveDate,
    pub monthly_income: f64,
    pub spending_limit_override: Option<f64>,
    /// Empty list means all cards are included; otherwise only specified card IDs
    #[serde(default)]
    pub linked_card_ids: Vec<Uuid>,
    pub created_at: NaiveDate,
    #[serde(default)]
    pub closed: bool,
    pub closed_at: Option<NaiveDate>,
    pub final_saved: Option<f64>,
    pub goal_met: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub cards: Vec<Card>,
    pub transactions: Vec<Transaction>,
    pub subscriptions: Vec<Subscription>,
    pub plans: Vec<SavingsPlan>,
    pub custom_categories: Vec<String>,
    pub last_checked_date: Option<NaiveDate>,
}

impl Default for AppData {
    fn default() -> Self {
        let default_card_id = Uuid::new_v4();
        let default_card = Card {
            id: default_card_id,
            name: "Main Card".to_string(),
            is_primary: true,
            opening_balance: 0.0,
            opening_balance_description: "Opening Balance".to_string(),
            opening_balance_date: None,
        };

        // Today or Sep 2026 default seed date
        let seed_date = chrono::Local::now().date_naive();

        // Pre-seed subscriptions as requested:
        // HBO MAX - Yearly - Rs 8,200 - started September 2026
        // Spotify - Monthly - Rs 400
        let hbo_start = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap_or(seed_date);
        let spotify_start = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap_or(seed_date);

        let hbo_next = hbo_start
            .checked_add_months(chrono::Months::new(12))
            .unwrap_or(hbo_start);
        let spotify_next = spotify_start
            .checked_add_months(chrono::Months::new(1))
            .unwrap_or(spotify_start);

        let hbo = Subscription {
            id: Uuid::new_v4(),
            card_id: default_card_id,
            name: "HBO MAX".to_string(),
            amount: 8200.0,
            cycle: BillingCycle::Yearly,
            start_date: hbo_start,
            next_due_date: hbo_next,
            paused: false,
        };

        let spotify = Subscription {
            id: Uuid::new_v4(),
            card_id: default_card_id,
            name: "Spotify".to_string(),
            amount: 400.0,
            cycle: BillingCycle::Monthly,
            start_date: spotify_start,
            next_due_date: spotify_next,
            paused: false,
        };

        Self {
            cards: vec![default_card],
            transactions: Vec::new(),
            subscriptions: vec![hbo, spotify],
            plans: Vec::new(),
            custom_categories: vec![
                "Food".to_string(),
                "Income".to_string(),
                "Subscription".to_string(),
                "Transport".to_string(),
                "Shopping".to_string(),
                "Other".to_string(),
            ],
            last_checked_date: Some(seed_date),
        }
    }
}
