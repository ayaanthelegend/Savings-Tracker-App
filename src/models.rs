use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn default_uuid() -> Uuid {
    Uuid::new_v4()
}

pub fn default_chrono_now() -> DateTime<Utc> {
    Utc::now()
}

pub fn default_opening_balance_description() -> String {
    "Opening Balance".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Card {
    #[serde(default = "default_uuid")]
    pub id: Uuid,
    pub name: String,
    pub is_primary: bool,
    pub opening_balance: f64,
    #[serde(default = "default_opening_balance_description")]
    pub opening_balance_description: String,
    #[serde(default)]
    pub opening_balance_date: Option<NaiveDate>,
    #[serde(default = "default_chrono_now")]
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    #[serde(default = "default_uuid")]
    pub id: Uuid,
    pub card_id: Uuid,
    pub date: NaiveDate,
    pub description: String,
    pub category: String,
    pub amount: f64,
    pub is_income: bool,
    #[serde(default)]
    pub auto_generated: bool,
    #[serde(default = "default_chrono_now")]
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
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

    pub fn to_serialized_str(&self) -> String {
        match self {
            BillingCycle::Monthly => "monthly".to_string(),
            BillingCycle::Yearly => "yearly".to_string(),
            BillingCycle::CustomDays(days) => format!("custom:{}", days),
        }
    }

    pub fn from_serialized_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "monthly" => BillingCycle::Monthly,
            "yearly" => BillingCycle::Yearly,
            other => {
                if let Some(days_str) = other.strip_prefix("custom:") {
                    if let Ok(days) = days_str.parse::<u32>() {
                        return BillingCycle::CustomDays(days);
                    }
                }
                BillingCycle::Monthly
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subscription {
    #[serde(default = "default_uuid")]
    pub id: Uuid,
    pub card_id: Uuid,
    pub name: String,
    pub amount: f64,
    pub cycle: BillingCycle,
    pub start_date: NaiveDate,
    pub next_due_date: NaiveDate,
    #[serde(default)]
    pub paused: bool,
    #[serde(default = "default_chrono_now")]
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavingsPlan {
    #[serde(default = "default_uuid")]
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
    #[serde(default)]
    pub deduct_overspending: bool,
    #[serde(default = "default_chrono_now")]
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    #[serde(default = "default_uuid")]
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default = "default_chrono_now")]
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub cards: Vec<Card>,
    pub transactions: Vec<Transaction>,
    pub subscriptions: Vec<Subscription>,
    pub plans: Vec<SavingsPlan>,
    pub custom_categories: Vec<String>,
    #[serde(default)]
    pub categories: Vec<Category>,
    pub last_checked_date: Option<NaiveDate>,
}

impl AppData {
    /// Ensures that loaded data from disk has valid sync timestamps and that categories match custom_categories.
    pub fn ensure_sync_fields(&mut self) {
        let now = Utc::now();
        // Synchronize categories and custom_categories
        if self.categories.is_empty() && !self.custom_categories.is_empty() {
            let defaults = ["Food", "Income", "Subscription", "Transport", "Shopping", "Other"];
            for cat_name in &self.custom_categories {
                self.categories.push(Category {
                    id: Uuid::new_v4(),
                    name: cat_name.clone(),
                    is_default: defaults.iter().any(|d| d.eq_ignore_ascii_case(cat_name)),
                    updated_at: now,
                    deleted_at: None,
                });
            }
        } else {
            for cat in &self.categories {
                if cat.deleted_at.is_none() && !self.custom_categories.iter().any(|c| c.eq_ignore_ascii_case(&cat.name)) {
                    self.custom_categories.push(cat.name.clone());
                }
            }
        }
    }
}

impl Default for AppData {
    fn default() -> Self {
        let default_card_id = Uuid::new_v4();
        let now = Utc::now();
        let default_card = Card {
            id: default_card_id,
            name: "Main Card".to_string(),
            is_primary: true,
            opening_balance: 0.0,
            opening_balance_description: "Opening Balance".to_string(),
            opening_balance_date: None,
            updated_at: now,
            deleted_at: None,
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
            updated_at: now,
            deleted_at: None,
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
            updated_at: now,
            deleted_at: None,
        };

        let default_categories = vec![
            "Food".to_string(),
            "Income".to_string(),
            "Subscription".to_string(),
            "Transport".to_string(),
            "Shopping".to_string(),
            "Other".to_string(),
        ];

        let categories = default_categories
            .iter()
            .map(|name| Category {
                id: Uuid::new_v4(),
                name: name.clone(),
                is_default: true,
                updated_at: now,
                deleted_at: None,
            })
            .collect();

        Self {
            cards: vec![default_card],
            transactions: Vec::new(),
            subscriptions: vec![hbo, spotify],
            plans: Vec::new(),
            custom_categories: default_categories,
            categories,
            last_checked_date: Some(seed_date),
        }
    }
}
