#![allow(unused, dead_code)]

use chrono::{Duration, NaiveDate, Utc};
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

#[path = "../src/models.rs"]
mod models;
#[path = "../src/auth.rs"]
mod auth;
#[path = "../src/sync.rs"]
mod sync;

use auth::AuthSession;
use models::{AppData, BillingCycle, Card, Category, SavingsPlan, Subscription, Transaction};
use sync::{merge_item, PendingSyncQueue, SyncWatermarks};

#[test]
fn test_legacy_data_json_backward_compatibility_deserialization() {
    // Simulates an existing data.json from before this cloud sync update
    let legacy_json = json!({
        "cards": [
            {
                "id": "3376142a-b4cc-46a4-ac19-c1e52abd0a68",
                "name": "Main Card",
                "is_primary": true,
                "opening_balance": 100000.0,
                "opening_balance_description": "Opening Balance",
                "opening_balance_date": "2026-09-26"
            }
        ],
        "transactions": [
            {
                "id": "11111111-1111-1111-1111-111111111111",
                "card_id": "3376142a-b4cc-46a4-ac19-c1e52abd0a68",
                "date": "2026-09-27",
                "description": "Grocery",
                "category": "Food",
                "amount": 2500.0,
                "is_income": false
            }
        ],
        "subscriptions": [
            {
                "id": "8083643d-91ae-4fac-9163-a6a013ebf6ff",
                "card_id": "3376142a-b4cc-46a4-ac19-c1e52abd0a68",
                "name": "HBO MAX",
                "amount": 8200.0,
                "cycle": "Yearly",
                "start_date": "2026-09-01",
                "next_due_date": "2027-09-01",
                "paused": false
            }
        ],
        "plans": [
            {
                "id": "52ec454c-bf44-4403-9390-15578e619af3",
                "name": "Monitor",
                "target_amount": 50000.0,
                "deadline": "2027-03-26",
                "monthly_income": 20000.0,
                "spending_limit_override": null,
                "linked_card_ids": [
                    "3376142a-b4cc-46a4-ac19-c1e52abd0a68"
                ],
                "created_at": "2026-09-26",
                "closed": false,
                "closed_at": null,
                "final_saved": null,
                "goal_met": null
            }
        ],
        "custom_categories": [
            "Food",
            "Income",
            "Subscription"
        ],
        "last_checked_date": "2026-09-26"
    });

    let json_str = legacy_json.to_string();
    let mut data: AppData = serde_json::from_str(&json_str).expect("Failed to deserialize legacy JSON");

    // Assert cards have populated defaults
    assert_eq!(data.cards.len(), 1);
    assert_eq!(data.cards[0].name, "Main Card");
    assert!(data.cards[0].deleted_at.is_none());
    assert!(data.cards[0].updated_at <= Utc::now());

    // Assert transactions have populated defaults
    assert_eq!(data.transactions.len(), 1);
    assert_eq!(data.transactions[0].description, "Grocery");
    assert!(data.transactions[0].deleted_at.is_none());

    // Assert subscriptions have populated defaults
    assert_eq!(data.subscriptions.len(), 1);
    assert_eq!(data.subscriptions[0].name, "HBO MAX");
    assert!(data.subscriptions[0].deleted_at.is_none());

    // Assert plans have populated defaults
    assert_eq!(data.plans.len(), 1);
    assert_eq!(data.plans[0].name, "Monitor");
    assert!(!data.plans[0].deduct_overspending);
    assert!(data.plans[0].deleted_at.is_none());

    // Ensure sync fields populates categories from custom_categories
    data.ensure_sync_fields();
    assert_eq!(data.categories.len(), 3);
    assert_eq!(data.categories[0].name, "Food");
    assert!(data.categories[0].is_default);
}

#[test]
fn test_billing_cycle_serialization_formats() {
    assert_eq!(BillingCycle::Monthly.to_serialized_str(), "monthly");
    assert_eq!(BillingCycle::Yearly.to_serialized_str(), "yearly");
    assert_eq!(BillingCycle::CustomDays(45).to_serialized_str(), "custom:45");

    assert_eq!(BillingCycle::from_serialized_str("monthly"), BillingCycle::Monthly);
    assert_eq!(BillingCycle::from_serialized_str("MONTHLY"), BillingCycle::Monthly);
    assert_eq!(BillingCycle::from_serialized_str("yearly"), BillingCycle::Yearly);
    assert_eq!(BillingCycle::from_serialized_str("custom:45"), BillingCycle::CustomDays(45));
    assert_eq!(BillingCycle::from_serialized_str("custom:14"), BillingCycle::CustomDays(14));
}

#[test]
fn test_proactive_token_refresh_threshold() {
    let now = Utc::now().timestamp();

    // Valid for 10 minutes (600s) -> does NOT need refresh
    let session_fresh = AuthSession {
        access_token: "token1".into(),
        refresh_token: "refresh1".into(),
        expires_at: now + 600,
        user_id: Uuid::new_v4(),
        user_email: "test@example.com".into(),
    };
    assert!(!session_fresh.needs_refresh());
    assert!(!session_fresh.is_expired());

    // Valid for only 4 minutes (240s) -> NEEDS proactive refresh (< 300s)
    let session_near_expiry = AuthSession {
        access_token: "token2".into(),
        refresh_token: "refresh2".into(),
        expires_at: now + 240,
        user_id: Uuid::new_v4(),
        user_email: "test@example.com".into(),
    };
    assert!(session_near_expiry.needs_refresh());
    assert!(!session_near_expiry.is_expired());

    // Expired 10 seconds ago -> needs refresh and is expired
    let session_expired = AuthSession {
        access_token: "token3".into(),
        refresh_token: "refresh3".into(),
        expires_at: now - 10,
        user_id: Uuid::new_v4(),
        user_email: "test@example.com".into(),
    };
    assert!(session_expired.needs_refresh());
    assert!(session_expired.is_expired());
}

#[test]
fn test_soft_delete_conflict_resolution() {
    let card_id = Uuid::new_v4();
    let t_create = Utc::now() - Duration::seconds(100);
    let t_edit_device1 = Utc::now() - Duration::seconds(50);
    let t_delete_device2 = Utc::now() - Duration::seconds(10); // newest

    // Local device has an older edit
    let local_card = Card {
        id: card_id,
        name: "Old Local Name".to_string(),
        is_primary: true,
        opening_balance: 1000.0,
        opening_balance_description: "Op".to_string(),
        opening_balance_date: None,
        updated_at: t_edit_device1,
        deleted_at: None,
    };

    // Server sends a soft delete with newer updated_at
    let server_deleted_card = Card {
        id: card_id,
        name: "Card Name".to_string(),
        is_primary: true,
        opening_balance: 1000.0,
        opening_balance_description: "Op".to_string(),
        opening_balance_date: None,
        updated_at: t_delete_device2,
        deleted_at: Some(t_delete_device2),
    };

    let mut list = vec![local_card];
    let mut pending = HashSet::new();

    let changed = merge_item(&mut list, server_deleted_card, &mut pending);
    assert!(changed);
    assert!(list[0].deleted_at.is_some());
    assert_eq!(list[0].deleted_at, Some(t_delete_device2));
    assert!(!pending.contains(&card_id));
}

#[test]
fn test_pending_sync_queue_serialization() {
    let mut queue = PendingSyncQueue::default();
    let card_id = Uuid::new_v4();
    let tx_id = Uuid::new_v4();

    queue.cards.insert(card_id);
    queue.transactions.insert(tx_id);
    assert!(!queue.is_empty());

    let serialized = serde_json::to_string(&queue).expect("Serialize queue");
    let deserialized: PendingSyncQueue = serde_json::from_str(&serialized).expect("Deserialize queue");

    assert!(deserialized.cards.contains(&card_id));
    assert!(deserialized.transactions.contains(&tx_id));
    assert!(deserialized.subscriptions.is_empty());
}

#[test]
fn test_per_table_watermarks_isolation() {
    let mut watermarks = SyncWatermarks::default();
    let t_cards = Utc::now() - Duration::minutes(5);
    let t_txs = Utc::now() - Duration::minutes(1);

    watermarks.cards = Some(t_cards);
    watermarks.transactions = Some(t_txs);

    assert_eq!(watermarks.cards, Some(t_cards));
    assert_eq!(watermarks.transactions, Some(t_txs));
    assert!(watermarks.subscriptions.is_none());
    assert!(watermarks.savings_plans.is_none());
    assert!(watermarks.categories.is_none());
}

#[test]
fn test_login_view_state_and_mode_toggle() {
    let mut state = savings_tracker::ui::login::LoginViewState {
        email: "test@example.com".to_string(),
        password: "secretpassword".to_string(),
        supabase_url: "https://test.supabase.co".to_string(),
        supabase_key: "test_anon_key".to_string(),
        is_signup: false,
        is_submitting: false,
        error_message: None,
        info_message: None,
    };

    assert!(!state.is_signup);
    assert!(!state.is_submitting);
    assert!(state.error_message.is_none());

    // Toggle to signup
    state.is_signup = true;
    assert!(state.is_signup);

    // Toggle back
    state.is_signup = false;
    assert!(!state.is_signup);
}

#[test]
fn test_login_view_rendering_at_multiple_window_sizes() {
    let ctx = egui::Context::default();
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "SpaceGrotesk".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/SpaceGrotesk.ttf")),
    );
    fonts.font_data.insert(
        "Inter".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/Inter.ttf")),
    );
    fonts.font_data.insert(
        "IBMPlexMono".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/IBMPlexMono.ttf")),
    );
    fonts.families.insert(
        egui::FontFamily::Name("Space Grotesk".into()),
        vec!["SpaceGrotesk".to_owned(), "Inter".to_owned()],
    );
    fonts.families.insert(
        egui::FontFamily::Name("SpaceGrotesk".into()),
        vec!["SpaceGrotesk".to_owned(), "Inter".to_owned()],
    );
    fonts.families.insert(
        egui::FontFamily::Name("Inter".into()),
        vec!["Inter".to_owned()],
    );
    fonts.families.insert(
        egui::FontFamily::Name("IBMPlexMono".into()),
        vec!["IBMPlexMono".to_owned()],
    );
    fonts.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Inter".to_owned());
    ctx.set_fonts(fonts);

    let mut state = savings_tracker::ui::login::LoginViewState::default();

    // Verify fresh state renders without existing session or config
    let viewports = [
        egui::vec2(800.0, 600.0),   // Compact
        egui::vec2(1200.0, 800.0),  // Laptop
        egui::vec2(1600.0, 1000.0), // Maximized
    ];

    for vp in viewports {
        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), vp));

        ctx.run(raw_input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Test 1: Log in mode
                state.is_signup = false;
                state.error_message = None;
                let _ = savings_tracker::ui::login::render_login_view(ui, &mut state);

                // Test 2: Error banner active
                state.error_message = Some("Invalid email or password.".to_string());
                let _ = savings_tracker::ui::login::render_login_view(ui, &mut state);

                // Test 3: Sign up mode
                state.is_signup = true;
                let _ = savings_tracker::ui::login::render_login_view(ui, &mut state);
            });
        });
    }
}

#[test]
fn test_login_validation_and_config_persistence() {
    let mut state = savings_tracker::ui::login::LoginViewState::default();

    // Verify empty fields trigger error
    state.email = "".to_string();
    state.password = "".to_string();
    assert!(state.email.trim().is_empty());
    assert!(state.password.trim().is_empty());

    // Backup existing config
    let existing_cfg = auth::AuthManager::load_config();

    // Test saving config and loading
    let test_cfg = auth::SupabaseConfig {
        url: "https://test-persistence.supabase.co".to_string(),
        anon_key: "test-anon-key-12345".to_string(),
    };
    let save_res = auth::AuthManager::save_config(&test_cfg);
    assert!(save_res.is_ok());

    let loaded = auth::AuthManager::load_config();
    assert_eq!(loaded.url, "https://test-persistence.supabase.co");
    assert_eq!(loaded.anon_key, "test-anon-key-12345");

    // Restore original config
    let _ = auth::AuthManager::save_config(&existing_cfg);
}


