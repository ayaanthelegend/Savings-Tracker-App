use crate::auth::{AuthManager, AuthSession, SupabaseConfig};
use crate::models::{AppData, BillingCycle, Card, Category, SavingsPlan, Subscription, Transaction};
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

// --- SUPABASE REST ROW MODELS ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub is_primary: bool,
    pub opening_balance: f64,
    pub opening_balance_description: String,
    #[serde(default)]
    pub opening_balance_date: Option<NaiveDate>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CardRow {
    pub fn from_local(c: &Card, user_id: Uuid) -> Self {
        Self {
            id: c.id,
            user_id,
            name: c.name.clone(),
            is_primary: c.is_primary,
            opening_balance: c.opening_balance,
            opening_balance_description: c.opening_balance_description.clone(),
            opening_balance_date: c.opening_balance_date,
            updated_at: c.updated_at,
            deleted_at: c.deleted_at,
        }
    }

    pub fn to_local(&self) -> Card {
        Card {
            id: self.id,
            name: self.name.clone(),
            is_primary: self.is_primary,
            opening_balance: self.opening_balance,
            opening_balance_description: self.opening_balance_description.clone(),
            opening_balance_date: self.opening_balance_date,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub card_id: Uuid,
    pub date: NaiveDate,
    pub description: String,
    pub category: String,
    pub amount: f64,
    pub is_income: bool,
    pub auto_generated: bool,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}

impl TransactionRow {
    pub fn from_local(t: &Transaction, user_id: Uuid) -> Self {
        Self {
            id: t.id,
            user_id,
            card_id: t.card_id,
            date: t.date,
            description: t.description.clone(),
            category: t.category.clone(),
            amount: t.amount,
            is_income: t.is_income,
            auto_generated: t.auto_generated,
            updated_at: t.updated_at,
            deleted_at: t.deleted_at,
        }
    }

    pub fn to_local(&self) -> Transaction {
        Transaction {
            id: self.id,
            card_id: self.card_id,
            date: self.date,
            description: self.description.clone(),
            category: self.category.clone(),
            amount: self.amount,
            is_income: self.is_income,
            auto_generated: self.auto_generated,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub card_id: Uuid,
    pub name: String,
    pub amount: f64,
    pub billing_cycle: String,
    #[serde(default)]
    pub start_date: Option<NaiveDate>,
    pub next_due_date: NaiveDate,
    pub is_paused: bool,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}

impl SubscriptionRow {
    pub fn from_local(s: &Subscription, user_id: Uuid) -> Self {
        Self {
            id: s.id,
            user_id,
            card_id: s.card_id,
            name: s.name.clone(),
            amount: s.amount,
            billing_cycle: s.cycle.to_serialized_str(),
            start_date: Some(s.start_date),
            next_due_date: s.next_due_date,
            is_paused: s.paused,
            updated_at: s.updated_at,
            deleted_at: s.deleted_at,
        }
    }

    pub fn to_local(&self) -> Subscription {
        Subscription {
            id: self.id,
            card_id: self.card_id,
            name: self.name.clone(),
            amount: self.amount,
            cycle: BillingCycle::from_serialized_str(&self.billing_cycle),
            start_date: self.start_date.unwrap_or(self.next_due_date),
            next_due_date: self.next_due_date,
            paused: self.is_paused,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsPlanRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub target_amount: f64,
    pub deadline: NaiveDate,
    pub monthly_income: f64,
    #[serde(default)]
    pub spending_limit: Option<f64>,
    pub is_active: bool,
    #[serde(default)]
    pub deduct_overspending: bool,
    #[serde(default)]
    pub linked_card_ids: Option<Vec<Uuid>>,
    #[serde(default)]
    pub created_at: Option<NaiveDate>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}

impl SavingsPlanRow {
    pub fn from_local(p: &SavingsPlan, user_id: Uuid) -> Self {
        Self {
            id: p.id,
            user_id,
            name: p.name.clone(),
            target_amount: p.target_amount,
            deadline: p.deadline,
            monthly_income: p.monthly_income,
            spending_limit: p.spending_limit_override,
            is_active: !p.closed,
            deduct_overspending: p.deduct_overspending,
            linked_card_ids: Some(p.linked_card_ids.clone()),
            created_at: Some(p.created_at),
            updated_at: p.updated_at,
            deleted_at: p.deleted_at,
        }
    }

    pub fn to_local(&self) -> SavingsPlan {
        SavingsPlan {
            id: self.id,
            name: self.name.clone(),
            target_amount: self.target_amount,
            deadline: self.deadline,
            monthly_income: self.monthly_income,
            spending_limit_override: self.spending_limit,
            linked_card_ids: self.linked_card_ids.clone().unwrap_or_default(),
            created_at: self.created_at.unwrap_or(self.deadline),
            closed: !self.is_active,
            closed_at: None,
            final_saved: None,
            goal_met: None,
            deduct_overspending: self.deduct_overspending,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub is_default: bool,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CategoryRow {
    pub fn from_local(c: &Category, user_id: Uuid) -> Self {
        Self {
            id: c.id,
            user_id,
            name: c.name.clone(),
            is_default: c.is_default,
            updated_at: c.updated_at,
            deleted_at: c.deleted_at,
        }
    }

    pub fn to_local(&self) -> Category {
        Category {
            id: self.id,
            name: self.name.clone(),
            is_default: self.is_default,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        }
    }
}

// --- WATERMARKS & PENDING QUEUE ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncWatermarks {
    pub cards: Option<DateTime<Utc>>,
    pub transactions: Option<DateTime<Utc>>,
    pub subscriptions: Option<DateTime<Utc>>,
    pub savings_plans: Option<DateTime<Utc>>,
    pub categories: Option<DateTime<Utc>>,
}

impl SyncWatermarks {
    pub fn file_path() -> PathBuf {
        AuthManager::get_app_dir().join("sync_watermarks.json")
    }

    pub fn load() -> Self {
        let path = Self::file_path();
        if path.exists() {
            if let Ok(c) = fs::read_to_string(&path) {
                if let Ok(w) = serde_json::from_str::<SyncWatermarks>(&c) {
                    return w;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::file_path();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PendingSyncQueue {
    pub cards: HashSet<Uuid>,
    pub transactions: HashSet<Uuid>,
    pub subscriptions: HashSet<Uuid>,
    pub savings_plans: HashSet<Uuid>,
    pub categories: HashSet<Uuid>,
}

impl PendingSyncQueue {
    pub fn file_path() -> PathBuf {
        AuthManager::get_app_dir().join("pending_sync.json")
    }

    pub fn load() -> Self {
        let path = Self::file_path();
        if path.exists() {
            if let Ok(c) = fs::read_to_string(&path) {
                if let Ok(q) = serde_json::from_str::<PendingSyncQueue>(&c) {
                    return q;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::file_path();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
            && self.transactions.is_empty()
            && self.subscriptions.is_empty()
            && self.savings_plans.is_empty()
            && self.categories.is_empty()
    }
}

// --- SYNC EVENTS & STATUS ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Synced(DateTime<Utc>),
    Offline(String),
    AuthError(String),
}

#[derive(Debug)]
pub enum SyncCommand {
    EnqueuePush,
    TriggerPull,
    UpdateSession(AuthSession),
    Logout,
    CheckInitialSync,
    Shutdown,
}

#[derive(Debug)]
pub enum SyncEvent {
    DataMerged(Box<AppData>),
    PushCompleted,
    #[allow(dead_code)]
    StatusChanged(SyncStatus),
    SessionRefreshed(AuthSession),
    AuthFailed(String),
}

// --- SYNC ENGINE ---

pub struct SyncManager {
    cmd_tx: Sender<SyncCommand>,
    event_rx: Receiver<SyncEvent>,
    latest_status: Arc<Mutex<SyncStatus>>,
}

impl SyncManager {
    pub fn new(app_data: Arc<Mutex<AppData>>) -> Self {
        let (cmd_tx, cmd_rx) = channel::<SyncCommand>();
        let (event_tx, event_rx) = channel::<SyncEvent>();
        let latest_status = Arc::new(Mutex::new(SyncStatus::Idle));
        let latest_status_clone = Arc::clone(&latest_status);

        // Spawn background sync worker thread
        thread::spawn(move || {
            let mut worker = SyncWorker::new(app_data, cmd_rx, event_tx, latest_status_clone);
            worker.run();
        });

        Self {
            cmd_tx,
            event_rx,
            latest_status,
        }
    }

    pub fn enqueue_push(&self) {
        let _ = self.cmd_tx.send(SyncCommand::EnqueuePush);
    }

    pub fn trigger_pull(&self) {
        let _ = self.cmd_tx.send(SyncCommand::TriggerPull);
    }

    pub fn update_session(&self, session: AuthSession) {
        let _ = self.cmd_tx.send(SyncCommand::UpdateSession(session));
    }

    pub fn check_initial_sync(&self) {
        let _ = self.cmd_tx.send(SyncCommand::CheckInitialSync);
    }

    pub fn logout(&self) {
        let _ = self.cmd_tx.send(SyncCommand::Logout);
    }

    pub fn try_recv_event(&self) -> Option<SyncEvent> {
        self.event_rx.try_recv().ok()
    }

    pub fn current_status(&self) -> SyncStatus {
        self.latest_status.lock().unwrap().clone()
    }
}

impl Drop for SyncManager {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(SyncCommand::Shutdown);
    }
}

// --- BACKGROUND SYNC WORKER ---

struct SyncWorker {
    app_data: Arc<Mutex<AppData>>,
    cmd_rx: Receiver<SyncCommand>,
    event_tx: Sender<SyncEvent>,
    latest_status: Arc<Mutex<SyncStatus>>,

    client: reqwest::blocking::Client,
    config: SupabaseConfig,
    session: Option<AuthSession>,
    watermarks: SyncWatermarks,
    pending_queue: PendingSyncQueue,

    consecutive_failures: u32,
    next_retry_at: Option<Instant>,
}

impl SyncWorker {
    pub fn new(
        app_data: Arc<Mutex<AppData>>,
        cmd_rx: Receiver<SyncCommand>,
        event_tx: Sender<SyncEvent>,
        latest_status: Arc<Mutex<SyncStatus>>,
    ) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        let config = AuthManager::load_config();
        let session = AuthManager::load_session();
        let watermarks = SyncWatermarks::load();
        let pending_queue = PendingSyncQueue::load();

        Self {
            app_data,
            cmd_rx,
            event_tx,
            latest_status,
            client,
            config,
            session,
            watermarks,
            pending_queue,
            consecutive_failures: 0,
            next_retry_at: None,
        }
    }

    fn set_status(&mut self, status: SyncStatus) {
        *self.latest_status.lock().unwrap() = status.clone();
        let _ = self.event_tx.send(SyncEvent::StatusChanged(status));
    }

    /// Computes exponential backoff delay based on consecutive failures.
    /// Starting at 2s, doubling each attempt, capped at 60s, up to 10 attempts.
    fn calculate_backoff(failures: u32) -> Duration {
        if failures == 0 {
            Duration::from_secs(0)
        } else if failures >= 10 {
            Duration::from_secs(60)
        } else {
            let base = 2_u64;
            let secs = (base.saturating_pow(failures)).min(60);
            Duration::from_secs(secs)
        }
    }

    pub fn run(&mut self) {
        // Initial setup
        if self.session.is_some() {
            self.ensure_fresh_token();
            self.do_pull();
            self.enqueue_all_from_app_data();
            self.do_push();
        }

        let mut last_periodic_sync = Instant::now();

        loop {
            // Process commands with short timeout
            match self.cmd_rx.recv_timeout(Duration::from_millis(500)) {
                Ok(SyncCommand::Shutdown) => break,
                Ok(SyncCommand::UpdateSession(session)) => {
                    self.session = Some(session);
                    self.consecutive_failures = 0;
                    self.next_retry_at = None;
                    last_periodic_sync = Instant::now();
                    self.do_pull();
                    self.do_push();
                }
                Ok(SyncCommand::Logout) => {
                    self.session = None;
                    self.watermarks = SyncWatermarks::default();
                    let _ = self.watermarks.save();
                    self.pending_queue = PendingSyncQueue::default();
                    let _ = self.pending_queue.save();
                    self.set_status(SyncStatus::Idle);
                }
                Ok(SyncCommand::EnqueuePush) => {
                    // Collect modified items into pending queue
                    self.enqueue_all_from_app_data();
                    self.do_push();
                }
                Ok(SyncCommand::TriggerPull) => {
                    self.do_pull();
                    if !self.pending_queue.is_empty() {
                        self.do_push();
                    }
                }
                Ok(SyncCommand::CheckInitialSync) => {
                    self.handle_initial_sync();
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Check if pending retry is due
                    if let Some(retry_time) = self.next_retry_at {
                        if Instant::now() >= retry_time {
                            self.next_retry_at = None;
                            if !self.pending_queue.is_empty() {
                                self.do_push();
                            }
                        }
                    } else if self.session.is_some() && last_periodic_sync.elapsed() >= Duration::from_secs(300) {
                        // Automatic 5-minute background cloud sync
                        last_periodic_sync = Instant::now();
                        self.do_pull();
                        self.enqueue_all_from_app_data();
                        self.do_push();
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    }

    /// Populates the pending queue with all items currently in local AppData
    /// so they can be upserted to Supabase.
    fn enqueue_all_from_app_data(&mut self) {
        let data = self.app_data.lock().unwrap().clone();
        for c in &data.cards {
            self.pending_queue.cards.insert(c.id);
        }
        for cat in &data.categories {
            self.pending_queue.categories.insert(cat.id);
        }
        for s in &data.subscriptions {
            self.pending_queue.subscriptions.insert(s.id);
        }
        for p in &data.plans {
            self.pending_queue.savings_plans.insert(p.id);
        }
        for t in &data.transactions {
            self.pending_queue.transactions.insert(t.id);
        }
        let _ = self.pending_queue.save();
    }

    /// Refreshes token proactively if within 5 minutes of expiry.
    fn ensure_fresh_token(&mut self) -> bool {
        let Some(session) = &self.session else {
            return false;
        };

        if session.needs_refresh() {
            // Proactive refresh
            match AuthManager::refresh_session(&self.client, &self.config, &session.refresh_token) {
                Ok(new_session) => {
                    let _ = self.event_tx.send(SyncEvent::SessionRefreshed(new_session.clone()));
                    self.session = Some(new_session);
                    true
                }
                Err(crate::auth::AuthError::TokenRevoked(msg)) => {
                    self.set_status(SyncStatus::AuthError(msg.clone()));
                    let _ = self.event_tx.send(SyncEvent::AuthFailed(msg));
                    false
                }
                Err(crate::auth::AuthError::Network(msg)) => {
                    // If still has some validity left, continue offline/temporarily
                    if session.is_expired() {
                        self.set_status(SyncStatus::Offline(format!("Network offline, token expired: {}", msg)));
                        false
                    } else {
                        true
                    }
                }
                Err(e) => {
                    self.set_status(SyncStatus::Offline(e.to_string()));
                    false
                }
            }
        } else {
            true
        }
    }

    /// Checks if cloud account has 0 rows.
    /// If 0 rows: push all local data as initial seed.
    /// If > 0 rows: pull and merge into local data.
    fn handle_initial_sync(&mut self) {
        if !self.ensure_fresh_token() {
            return;
        }

        let Some(session) = self.session.clone() else {
            return;
        };

        self.set_status(SyncStatus::Syncing);
        let base_url = AuthManager::load_config().url;
        let base_url = base_url.trim_end_matches('/');

        // Check cards count
        let count_endpoint = format!("{}/rest/v1/cards?select=id&limit=1", base_url);
        let resp = self.client
            .get(&count_endpoint)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {}", session.access_token))
            .header("Prefer", "count=exact")
            .send();

        match resp {
            Ok(r) => {
                let is_zero = if let Some(range_hdr) = r.headers().get("content-range") {
                    let s = range_hdr.to_str().unwrap_or_default();
                    s.ends_with("/0")
                } else if let Ok(json_arr) = r.json::<Vec<serde_json::Value>>() {
                    json_arr.is_empty()
                } else {
                    false
                };

                if is_zero {
                    // Initial seed push: queue all local rows and push
                    let data = self.app_data.lock().unwrap().clone();
                    for c in &data.cards { self.pending_queue.cards.insert(c.id); }
                    for cat in &data.categories { self.pending_queue.categories.insert(cat.id); }
                    for p in &data.plans { self.pending_queue.savings_plans.insert(p.id); }
                    for s in &data.subscriptions { self.pending_queue.subscriptions.insert(s.id); }
                    for t in &data.transactions { self.pending_queue.transactions.insert(t.id); }
                    let _ = self.pending_queue.save();

                    self.do_push();
                } else {
                    // Cloud has data: clear watermarks and pull all
                    self.watermarks = SyncWatermarks::default();
                    self.do_pull();
                    self.enqueue_all_from_app_data();
                    self.do_push();
                }
            }
            Err(e) => {
                self.set_status(SyncStatus::Offline(format!("Initial sync check failed: {}", e)));
            }
        }
    }

    /// Pushes pending items for each table in order to respect foreign key constraints.
    pub fn do_push(&mut self) {
        if self.pending_queue.is_empty() {
            return;
        }

        if !self.ensure_fresh_token() {
            return;
        }

        let Some(session) = self.session.clone() else {
            return;
        };

        self.set_status(SyncStatus::Syncing);
        let base_url = AuthManager::load_config().url;
        let base_url = base_url.trim_end_matches('/').to_string();
        let anon_key = self.config.anon_key.clone();

        let local_data = self.app_data.lock().unwrap().clone();
        let user_id = session.user_id;

        // Push order: cards, categories, savings_plans, subscriptions, transactions
        let mut any_push_succeeded = false;
        let mut any_push_failed = false;

        // 1. Cards
        if !self.pending_queue.cards.is_empty() {
            let rows: Vec<CardRow> = local_data
                .cards
                .iter()
                .filter(|c| self.pending_queue.cards.contains(&c.id))
                .map(|c| CardRow::from_local(c, user_id))
                .collect();

            if !rows.is_empty() {
                match self.push_table::<CardRow>(&base_url, &anon_key, &session.access_token, "cards", &rows) {
                    Ok(_) => {
                        self.pending_queue.cards.clear();
                        any_push_succeeded = true;
                    }
                    Err(e) => {
                        eprintln!("Push cards failed: {}", e);
                        any_push_failed = true;
                    }
                }
            } else {
                self.pending_queue.cards.clear();
            }
        }

        // 2. Categories
        if !self.pending_queue.categories.is_empty() {
            let rows: Vec<CategoryRow> = local_data
                .categories
                .iter()
                .filter(|c| self.pending_queue.categories.contains(&c.id))
                .map(|c| CategoryRow::from_local(c, user_id))
                .collect();

            if !rows.is_empty() {
                match self.push_table::<CategoryRow>(&base_url, &anon_key, &session.access_token, "categories", &rows) {
                    Ok(_) => {
                        self.pending_queue.categories.clear();
                        any_push_succeeded = true;
                    }
                    Err(e) => {
                        eprintln!("Push categories failed: {}", e);
                        any_push_failed = true;
                    }
                }
            } else {
                self.pending_queue.categories.clear();
            }
        }

        // 3. Savings Plans
        if !self.pending_queue.savings_plans.is_empty() {
            let rows: Vec<SavingsPlanRow> = local_data
                .plans
                .iter()
                .filter(|p| self.pending_queue.savings_plans.contains(&p.id))
                .map(|p| SavingsPlanRow::from_local(p, user_id))
                .collect();

            if !rows.is_empty() {
                match self.push_table::<SavingsPlanRow>(&base_url, &anon_key, &session.access_token, "savings_plans", &rows) {
                    Ok(_) => {
                        self.pending_queue.savings_plans.clear();
                        any_push_succeeded = true;
                    }
                    Err(e) => {
                        eprintln!("Push savings_plans failed: {}", e);
                        any_push_failed = true;
                    }
                }
            } else {
                self.pending_queue.savings_plans.clear();
            }
        }

        // 4. Subscriptions
        if !self.pending_queue.subscriptions.is_empty() {
            let rows: Vec<SubscriptionRow> = local_data
                .subscriptions
                .iter()
                .filter(|s| self.pending_queue.subscriptions.contains(&s.id))
                .map(|s| SubscriptionRow::from_local(s, user_id))
                .collect();

            if !rows.is_empty() {
                match self.push_table::<SubscriptionRow>(&base_url, &anon_key, &session.access_token, "subscriptions", &rows) {
                    Ok(_) => {
                        self.pending_queue.subscriptions.clear();
                        any_push_succeeded = true;
                    }
                    Err(e) => {
                        eprintln!("Push subscriptions failed: {}", e);
                        any_push_failed = true;
                    }
                }
            } else {
                self.pending_queue.subscriptions.clear();
            }
        }

        // 5. Transactions
        if !self.pending_queue.transactions.is_empty() {
            let rows: Vec<TransactionRow> = local_data
                .transactions
                .iter()
                .filter(|t| self.pending_queue.transactions.contains(&t.id))
                .map(|t| TransactionRow::from_local(t, user_id))
                .collect();

            if !rows.is_empty() {
                match self.push_table::<TransactionRow>(&base_url, &anon_key, &session.access_token, "transactions", &rows) {
                    Ok(_) => {
                        self.pending_queue.transactions.clear();
                        any_push_succeeded = true;
                    }
                    Err(e) => {
                        eprintln!("Push transactions failed: {}", e);
                        any_push_failed = true;
                    }
                }
            } else {
                self.pending_queue.transactions.clear();
            }
        }

        let _ = self.pending_queue.save();

        if any_push_failed {
            self.consecutive_failures += 1;
            let backoff = Self::calculate_backoff(self.consecutive_failures);
            if self.consecutive_failures < 10 {
                self.next_retry_at = Some(Instant::now() + backoff);
            } else {
                // Parked in pending queue until next regular tick
                self.next_retry_at = None;
            }
            self.set_status(SyncStatus::Offline(format!(
                "Push pending (retry {} after {:.0}s)",
                self.consecutive_failures,
                backoff.as_secs()
            )));
        } else {
            self.consecutive_failures = 0;
            self.next_retry_at = None;
            if any_push_succeeded || self.pending_queue.is_empty() {
                let _ = self.event_tx.send(SyncEvent::PushCompleted);
                self.set_status(SyncStatus::Synced(Utc::now()));
            }
        }
    }

    /// Upserts a batch of rows to Supabase PostgREST.
    fn push_table<T: Serialize>(
        &self,
        base_url: &str,
        anon_key: &str,
        access_token: &str,
        table_name: &str,
        rows: &[T],
    ) -> Result<DateTime<Utc>> {
        let endpoint = format!("{}/rest/v1/{}?on_conflict=id", base_url, table_name);

        let resp = self
            .client
            .post(&endpoint)
            .header("apikey", anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Prefer", "resolution=merge-duplicates,return=representation")
            .header("Content-Type", "application/json")
            .json(rows)
            .send()
            .context("Failed to send push request")?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().unwrap_or_default();
            anyhow::bail!("PostgREST push error {}: {}", status, err_text);
        }

        Ok(Utc::now())
    }

    /// Pulls updated rows from Supabase for all tables and merges them using last-write-wins.
    pub fn do_pull(&mut self) {
        if !self.ensure_fresh_token() {
            return;
        }

        let Some(session) = self.session.clone() else {
            return;
        };

        self.set_status(SyncStatus::Syncing);
        let base_url = AuthManager::load_config().url;
        let base_url = base_url.trim_end_matches('/').to_string();
        let anon_key = self.config.anon_key.clone();
        let access_token = session.access_token.clone();

        let mut local_data = self.app_data.lock().unwrap().clone();
        let mut data_changed = false;
        let mut pull_failed = false;

        // 1. Cards
        match self.pull_table::<CardRow>(&base_url, &anon_key, &access_token, "cards", self.watermarks.cards) {
            Ok((rows, max_updated)) => {
                for r in rows {
                    let server_item = r.to_local();
                    if merge_item(&mut local_data.cards, server_item, &mut self.pending_queue.cards) {
                        data_changed = true;
                    }
                }
                if let Some(mu) = max_updated {
                    self.watermarks.cards = Some(mu.max(self.watermarks.cards.unwrap_or(mu)));
                }
            }
            Err(e) => {
                eprintln!("Pull cards failed: {}", e);
                pull_failed = true;
            }
        }

        // 2. Categories
        match self.pull_table::<CategoryRow>(&base_url, &anon_key, &access_token, "categories", self.watermarks.categories) {
            Ok((rows, max_updated)) => {
                for r in rows {
                    let server_item = r.to_local();
                    if merge_item(&mut local_data.categories, server_item, &mut self.pending_queue.categories) {
                        data_changed = true;
                    }
                }
                if let Some(mu) = max_updated {
                    self.watermarks.categories = Some(mu.max(self.watermarks.categories.unwrap_or(mu)));
                }
            }
            Err(e) => {
                eprintln!("Pull categories failed: {}", e);
                pull_failed = true;
            }
        }

        // 3. Subscriptions
        match self.pull_table::<SubscriptionRow>(&base_url, &anon_key, &access_token, "subscriptions", self.watermarks.subscriptions) {
            Ok((rows, max_updated)) => {
                for r in rows {
                    let server_item = r.to_local();
                    if merge_item(&mut local_data.subscriptions, server_item, &mut self.pending_queue.subscriptions) {
                        data_changed = true;
                    }
                }
                if let Some(mu) = max_updated {
                    self.watermarks.subscriptions = Some(mu.max(self.watermarks.subscriptions.unwrap_or(mu)));
                }
            }
            Err(e) => {
                eprintln!("Pull subscriptions failed: {}", e);
                pull_failed = true;
            }
        }

        // 4. Savings Plans
        match self.pull_table::<SavingsPlanRow>(&base_url, &anon_key, &access_token, "savings_plans", self.watermarks.savings_plans) {
            Ok((rows, max_updated)) => {
                for r in rows {
                    let server_item = r.to_local();
                    if merge_item(&mut local_data.plans, server_item, &mut self.pending_queue.savings_plans) {
                        data_changed = true;
                    }
                }
                if let Some(mu) = max_updated {
                    self.watermarks.savings_plans = Some(mu.max(self.watermarks.savings_plans.unwrap_or(mu)));
                }
            }
            Err(e) => {
                eprintln!("Pull savings plans failed: {}", e);
                pull_failed = true;
            }
        }

        // 5. Transactions
        match self.pull_table::<TransactionRow>(&base_url, &anon_key, &access_token, "transactions", self.watermarks.transactions) {
            Ok((rows, max_updated)) => {
                for r in rows {
                    let server_item = r.to_local();
                    if merge_item(&mut local_data.transactions, server_item, &mut self.pending_queue.transactions) {
                        data_changed = true;
                    }
                }
                if let Some(mu) = max_updated {
                    self.watermarks.transactions = Some(mu.max(self.watermarks.transactions.unwrap_or(mu)));
                }
            }
            Err(e) => {
                eprintln!("Pull transactions failed: {}", e);
                pull_failed = true;
            }
        }

        let _ = self.watermarks.save();
        let _ = self.pending_queue.save();

        if data_changed {
            local_data.ensure_sync_fields();
            *self.app_data.lock().unwrap() = local_data.clone();
            let _ = self.event_tx.send(SyncEvent::DataMerged(Box::new(local_data)));
        }

        if pull_failed {
            self.consecutive_failures += 1;
            let backoff = Self::calculate_backoff(self.consecutive_failures);
            if self.consecutive_failures < 10 {
                self.next_retry_at = Some(Instant::now() + backoff);
            }
            self.set_status(SyncStatus::Offline(format!(
                "Pull error (retry {} in {:.0}s)",
                self.consecutive_failures,
                backoff.as_secs()
            )));
        } else {
            self.consecutive_failures = 0;
            self.next_retry_at = None;
            self.set_status(SyncStatus::Synced(Utc::now()));
        }
    }

    /// Fetches rows from a Supabase table with optional watermark filter.
    fn pull_table<T: for<'de> Deserialize<'de> + HasUpdatedAt>(
        &self,
        base_url: &str,
        anon_key: &str,
        access_token: &str,
        table_name: &str,
        watermark: Option<DateTime<Utc>>,
    ) -> Result<(Vec<T>, Option<DateTime<Utc>>)> {
        let mut endpoint = format!("{}/rest/v1/{}?select=*&order=updated_at.asc", base_url, table_name);
        if let Some(wm) = watermark {
            let iso_z = wm.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
            endpoint.push_str(&format!("&updated_at=gt.{}", iso_z));
        }

        let resp = self
            .client
            .get(&endpoint)
            .header("apikey", anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .context("Failed to send pull request")?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().unwrap_or_default();
            anyhow::bail!("PostgREST pull error {}: {}", status, err_text);
        }

        let rows: Vec<T> = resp.json().context("Failed to parse pull response JSON")?;
        let max_updated = rows.iter().map(|r| r.get_updated_at()).max();

        Ok((rows, max_updated))
    }
}

pub trait HasIdAndUpdated {
    fn get_id(&self) -> Uuid;
    fn get_updated_at(&self) -> DateTime<Utc>;
}

pub trait HasUpdatedAt {
    fn get_updated_at(&self) -> DateTime<Utc>;
}

impl HasUpdatedAt for CardRow { fn get_updated_at(&self) -> DateTime<Utc> { self.updated_at } }
impl HasUpdatedAt for TransactionRow { fn get_updated_at(&self) -> DateTime<Utc> { self.updated_at } }
impl HasUpdatedAt for SubscriptionRow { fn get_updated_at(&self) -> DateTime<Utc> { self.updated_at } }
impl HasUpdatedAt for SavingsPlanRow { fn get_updated_at(&self) -> DateTime<Utc> { self.updated_at } }
impl HasUpdatedAt for CategoryRow { fn get_updated_at(&self) -> DateTime<Utc> { self.updated_at } }

impl HasIdAndUpdated for Card {
    fn get_id(&self) -> Uuid { self.id }
    fn get_updated_at(&self) -> DateTime<Utc> { self.updated_at }
}
impl HasIdAndUpdated for Transaction {
    fn get_id(&self) -> Uuid { self.id }
    fn get_updated_at(&self) -> DateTime<Utc> { self.updated_at }
}
impl HasIdAndUpdated for Subscription {
    fn get_id(&self) -> Uuid { self.id }
    fn get_updated_at(&self) -> DateTime<Utc> { self.updated_at }
}
impl HasIdAndUpdated for SavingsPlan {
    fn get_id(&self) -> Uuid { self.id }
    fn get_updated_at(&self) -> DateTime<Utc> { self.updated_at }
}
impl HasIdAndUpdated for Category {
    fn get_id(&self) -> Uuid { self.id }
    fn get_updated_at(&self) -> DateTime<Utc> { self.updated_at }
}

/// Last-Write-Wins Merge Rule:
/// - If server_item.updated_at >= local_item.updated_at: server wins (including exact timestamp tie)!
/// - If local_item.updated_at > server_item.updated_at: local wins! Keep local, queue local ID for push.
/// Returns true if local list was modified.
pub fn merge_item<T: HasIdAndUpdated + Clone + PartialEq>(
    local_list: &mut Vec<T>,
    server_item: T,
    pending_pushes: &mut HashSet<Uuid>,
) -> bool {
    let server_id = server_item.get_id();
    let server_updated = server_item.get_updated_at();

    if let Some(pos) = local_list.iter().position(|item| item.get_id() == server_id) {
        let local_updated = local_list[pos].get_updated_at();
        // Server wins if >= (tie goes to server)
        if server_updated >= local_updated {
            if local_list[pos] != server_item {
                local_list[pos] = server_item;
                pending_pushes.remove(&server_id);
                return true;
            }
        } else {
            // Local is newer! Local wins, mark for push to server
            pending_pushes.insert(server_id);
        }
        false
    } else {
        // Not present locally: insert server item
        local_list.push(server_item);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_resolution_server_wins_on_newer_or_tie() {
        let id = Uuid::new_v4();
        let t1 = Utc::now();
        let t2 = t1 + chrono::Duration::seconds(10);

        let local_card = Card {
            id,
            name: "Local Card".to_string(),
            is_primary: true,
            opening_balance: 100.0,
            opening_balance_description: "Op".to_string(),
            opening_balance_date: None,
            updated_at: t1,
            deleted_at: None,
        };

        let server_card_newer = Card {
            id,
            name: "Server Card Newer".to_string(),
            is_primary: true,
            opening_balance: 200.0,
            opening_balance_description: "Op".to_string(),
            opening_balance_date: None,
            updated_at: t2,
            deleted_at: None,
        };

        let mut list = vec![local_card.clone()];
        let mut pending = HashSet::new();

        // Server newer -> server wins
        let changed = merge_item(&mut list, server_card_newer.clone(), &mut pending);
        assert!(changed);
        assert_eq!(list[0].name, "Server Card Newer");
        assert!(!pending.contains(&id));

        // Exact tie -> server wins
        let server_card_tie = Card {
            id,
            name: "Server Card Tie".to_string(),
            is_primary: true,
            opening_balance: 300.0,
            opening_balance_description: "Op".to_string(),
            opening_balance_date: None,
            updated_at: t2, // same as current list[0]
            deleted_at: None,
        };
        let changed_tie = merge_item(&mut list, server_card_tie, &mut pending);
        assert!(changed_tie);
        assert_eq!(list[0].name, "Server Card Tie");
    }

    #[test]
    fn test_conflict_resolution_local_wins_if_newer() {
        let id = Uuid::new_v4();
        let t_old = Utc::now() - chrono::Duration::seconds(60);
        let t_new = Utc::now();

        let local_card_newer = Card {
            id,
            name: "Local Card Newer".to_string(),
            is_primary: true,
            opening_balance: 500.0,
            opening_balance_description: "Op".to_string(),
            opening_balance_date: None,
            updated_at: t_new,
            deleted_at: None,
        };

        let server_card_older = Card {
            id,
            name: "Server Card Stale".to_string(),
            is_primary: true,
            opening_balance: 100.0,
            opening_balance_description: "Op".to_string(),
            opening_balance_date: None,
            updated_at: t_old,
            deleted_at: None,
        };

        let mut list = vec![local_card_newer.clone()];
        let mut pending = HashSet::new();

        let changed = merge_item(&mut list, server_card_older, &mut pending);
        assert!(!changed);
        assert_eq!(list[0].name, "Local Card Newer");
        // Must queue for push to overwrite server!
        assert!(pending.contains(&id));
    }

    #[test]
    fn test_backoff_calculation() {
        assert_eq!(SyncWorker::calculate_backoff(0), Duration::from_secs(0));
        assert_eq!(SyncWorker::calculate_backoff(1), Duration::from_secs(2));
        assert_eq!(SyncWorker::calculate_backoff(2), Duration::from_secs(4));
        assert_eq!(SyncWorker::calculate_backoff(3), Duration::from_secs(8));
        assert_eq!(SyncWorker::calculate_backoff(4), Duration::from_secs(16));
        assert_eq!(SyncWorker::calculate_backoff(5), Duration::from_secs(32));
        assert_eq!(SyncWorker::calculate_backoff(6), Duration::from_secs(60)); // capped at 60s
        assert_eq!(SyncWorker::calculate_backoff(10), Duration::from_secs(60));
    }
}
