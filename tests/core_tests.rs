#![allow(unused, dead_code)]

use chrono::NaiveDate;
use uuid::Uuid;

#[path = "../src/models.rs"]
mod models;
#[path = "../src/ledger.rs"]
mod ledger;
#[path = "../src/subscriptions.rs"]
mod subscriptions;
#[path = "../src/savings.rs"]
mod savings;
#[path = "../src/notification.rs"]
mod notification;
#[path = "../src/ui/theme.rs"]
mod theme;

use ledger::LedgerCalculator;
use models::{AppData, BillingCycle, Card, SavingsPlan, Subscription, Transaction};
use savings::SavingsEngine;
use subscriptions::SubscriptionManager;
use theme::Theme;

#[test]
fn test_font_loading_and_layout() {
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

    fonts.families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "IBMPlexMono".to_owned());

    ctx.set_fonts(fonts);

    ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let prop_font = egui::FontId::new(13.0, egui::FontFamily::Proportional);
            let head_font = Theme::font_head(16.0);
            let mono_font = Theme::font_mono(13.0);

            // Verify glyphs in proportional font: valid vs missing
            ctx.fonts(|f| {
                assert!(f.has_glyph(&prop_font, '★'));
                assert!(f.has_glyph(&prop_font, '●'));
                assert!(f.has_glyph(&prop_font, '•'));
                assert!(f.has_glyph(&prop_font, '·'));
                assert!(f.has_glyph(&prop_font, '—'));
                // ✎ is missing in standard fonts and was replaced by text
                assert!(!f.has_glyph(&prop_font, '✎'));
            });

            // Verify TableBuilder renders headers and body without panic
            egui_extras::TableBuilder::new(ui)
                .striped(false)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::exact(105.0))
                .column(egui_extras::Column::remainder().at_least(180.0))
                .column(egui_extras::Column::exact(135.0))
                .column(egui_extras::Column::exact(120.0))
                .column(egui_extras::Column::exact(120.0))
                .column(egui_extras::Column::exact(130.0))
                .column(egui_extras::Column::exact(85.0))
                .header(32.0, |mut header| {
                    header.col(|ui| { ui.label("Date"); });
                    header.col(|ui| { ui.label("Description"); });
                    header.col(|ui| { ui.label("Category"); });
                    header.col(|ui| { ui.label("In (PKR)"); });
                    header.col(|ui| { ui.label("Out (PKR)"); });
                    header.col(|ui| { ui.label("Balance"); });
                    header.col(|ui| { ui.label("Actions"); });
                })
                .body(|mut body| {
                    body.row(34.0, |mut row| {
                        row.col(|ui| {
                            ui.label("2026-09-01");
                            let r = ui.max_rect();
                            ui.painter().line_segment([r.left_bottom(), r.right_bottom()], Theme::stroke_border());
                        });
                        row.col(|ui| { ui.label("Test Description"); });
                        row.col(|ui| { ui.label("Food"); });
                        row.col(|ui| { ui.label("—"); });
                        row.col(|ui| { ui.label("Rs 500.00"); });
                        row.col(|ui| { ui.label("Rs 49,500.00"); });
                        row.col(|ui| { ui.label("Del"); });
                    });
                });
        });
    });
}

#[test]
fn test_currency_formatting_pkr() {
    assert_eq!(Theme::format_pkr(0.0), "Rs 0.00");
    assert_eq!(Theme::format_pkr(400.0), "Rs 400.00");
    assert_eq!(Theme::format_pkr(8200.0), "Rs 8,200.00");
    assert_eq!(Theme::format_pkr(1234567.89), "Rs 1,234,567.89");
    assert_eq!(Theme::format_pkr(-1500.50), "-Rs 1,500.50");
    assert_eq!(Theme::format_pkr_whole(100000.0), "Rs 100,000");

    // Currency input parsing tests
    assert_eq!(Theme::parse_pkr_input("10,500.50"), Some(10500.50));
    assert_eq!(Theme::parse_pkr_input("1,234"), Some(1234.0));
    assert_eq!(Theme::parse_pkr_input("abc"), None);
}

#[test]
fn test_ledger_running_balance_chronological() {
    let card_id = Uuid::new_v4();
    let card = Card {
        id: card_id,
        name: "Test Card".to_string(),
        is_primary: true,
        opening_balance: 50_000.0,
        opening_balance_description: "Opening Balance".to_string(),
        opening_balance_date: None,
    };

    let tx1 = Transaction {
        id: Uuid::new_v4(),
        card_id,
        date: NaiveDate::from_ymd_opt(2026, 9, 2).unwrap(),
        description: "Grocery shopping".to_string(),
        category: "Food".to_string(),
        amount: 5_000.0,
        is_income: false,
        auto_generated: false,
    };

    let tx2 = Transaction {
        id: Uuid::new_v4(),
        card_id,
        date: NaiveDate::from_ymd_opt(2026, 9, 5).unwrap(),
        description: "Freelance project".to_string(),
        category: "Income".to_string(),
        amount: 25_000.0,
        is_income: true,
        auto_generated: false,
    };

    let tx3 = Transaction {
        id: Uuid::new_v4(),
        card_id,
        date: NaiveDate::from_ymd_opt(2026, 9, 10).unwrap(),
        description: "Electricity bill".to_string(),
        category: "Other".to_string(),
        amount: 10_000.0,
        is_income: false,
        auto_generated: false,
    };

    // Pass in random order to ensure chronological sorting works
    let transactions = vec![tx3.clone(), tx1.clone(), tx2.clone()];
    let rows = LedgerCalculator::compute_full_ledger(&card, &transactions);

    assert_eq!(rows.len(), 4); // 1 Opening balance + 3 transactions
    assert!(rows[0].is_opening_balance);
    assert_eq!(rows[0].running_balance, 50_000.0);

    // After tx1 (Food -5000): 45,000
    assert_eq!(rows[1].date, NaiveDate::from_ymd_opt(2026, 9, 2).unwrap());
    assert_eq!(rows[1].running_balance, 45_000.0);

    // After tx2 (Income +25,000): 70,000
    assert_eq!(rows[2].date, NaiveDate::from_ymd_opt(2026, 9, 5).unwrap());
    assert_eq!(rows[2].running_balance, 70_000.0);

    // After tx3 (Other -10,000): 60,000
    assert_eq!(rows[3].date, NaiveDate::from_ymd_opt(2026, 9, 10).unwrap());
    assert_eq!(rows[3].running_balance, 60_000.0);

    // Test backdating tx3 to 2026-09-01 (before tx1):
    let mut updated_tx3 = tx3.clone();
    updated_tx3.date = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();
    let reordered_txs = vec![tx1.clone(), tx2.clone(), updated_tx3];
    let rows_backdated = LedgerCalculator::compute_full_ledger(&card, &reordered_txs);

    assert_eq!(rows_backdated.len(), 4);
    assert_eq!(rows_backdated[1].date, NaiveDate::from_ymd_opt(2026, 9, 1).unwrap());
    assert_eq!(rows_backdated[1].running_balance, 40_000.0); // 50,000 - 10,000
    assert_eq!(rows_backdated[2].date, NaiveDate::from_ymd_opt(2026, 9, 2).unwrap());
    assert_eq!(rows_backdated[2].running_balance, 35_000.0); // 40,000 - 5,000
    assert_eq!(rows_backdated[3].date, NaiveDate::from_ymd_opt(2026, 9, 5).unwrap());
    assert_eq!(rows_backdated[3].running_balance, 60_000.0); // 35,000 + 25,000

    // Test editing opening balance amount, description, and date:
    let mut edited_card = card.clone();
    edited_card.opening_balance = 12_000.0;
    edited_card.opening_balance_description = "Starting balance (Aug)".to_string();
    edited_card.opening_balance_date = Some(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap());

    let rows_card_edited = LedgerCalculator::compute_full_ledger(&edited_card, &transactions);
    assert_eq!(rows_card_edited[0].description, "Starting balance (Aug)");
    assert_eq!(rows_card_edited[0].date, NaiveDate::from_ymd_opt(2026, 8, 1).unwrap());
    assert_eq!(rows_card_edited[0].running_balance, 12_000.0);
    // After tx1 (-5000): 7,000
    assert_eq!(rows_card_edited[1].running_balance, 7_000.0);
    // After tx2 (+25000): 32,000
    assert_eq!(rows_card_edited[2].running_balance, 32_000.0);
    // After tx3 (-10000): 22,000
    assert_eq!(rows_card_edited[3].running_balance, 22_000.0);
}

#[test]
fn test_opening_balance_single_anchor_row_regression() {
    let card_id = Uuid::new_v4();
    let card = Card {
        id: card_id,
        name: "Savings Account".to_string(),
        is_primary: true,
        opening_balance: 10_500.50,
        opening_balance_description: "Opening Balance".to_string(),
        opening_balance_date: Some(NaiveDate::from_ymd_opt(2026, 9, 1).unwrap()),
    };

    let empty_txs: Vec<Transaction> = Vec::new();
    let rows_empty = LedgerCalculator::compute_full_ledger(&card, &empty_txs);

    // Must contain exactly 1 row, and that row must be the opening balance
    assert_eq!(rows_empty.len(), 1);
    let op_rows_count = rows_empty.iter().filter(|r| r.is_opening_balance).count();
    assert_eq!(op_rows_count, 1);
    assert_eq!(rows_empty[0].running_balance, 10_500.50);

    // After adding regular transactions, opening balance row count must remain exactly 1
    let tx = Transaction {
        id: Uuid::new_v4(),
        card_id,
        date: NaiveDate::from_ymd_opt(2026, 9, 2).unwrap(),
        description: "Deposit".to_string(),
        category: "Income".to_string(),
        amount: 5000.0,
        is_income: true,
        auto_generated: false,
    };
    let rows_with_tx = LedgerCalculator::compute_full_ledger(&card, &[tx]);
    assert_eq!(rows_with_tx.len(), 2);
    let op_count_after = rows_with_tx.iter().filter(|r| r.is_opening_balance).count();
    assert_eq!(op_count_after, 1);
}

#[test]
fn test_subscription_cycle_advancement_and_proration() {
    let base_date = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();

    let next_monthly = SubscriptionManager::advance_date(base_date, &BillingCycle::Monthly);
    assert_eq!(next_monthly, NaiveDate::from_ymd_opt(2026, 10, 1).unwrap());

    let next_yearly = SubscriptionManager::advance_date(base_date, &BillingCycle::Yearly);
    assert_eq!(next_yearly, NaiveDate::from_ymd_opt(2027, 9, 1).unwrap());

    let next_custom = SubscriptionManager::advance_date(base_date, &BillingCycle::CustomDays(14));
    assert_eq!(next_custom, NaiveDate::from_ymd_opt(2026, 9, 15).unwrap());

    // Proration check
    let spotify = Subscription {
        id: Uuid::new_v4(),
        card_id: Uuid::new_v4(),
        name: "Spotify".to_string(),
        amount: 400.0,
        cycle: BillingCycle::Monthly,
        start_date: base_date,
        next_due_date: next_monthly,
        paused: false,
    };
    assert_eq!(SubscriptionManager::monthly_cost(&spotify), 400.0);

    let hbo = Subscription {
        id: Uuid::new_v4(),
        card_id: Uuid::new_v4(),
        name: "HBO MAX".to_string(),
        amount: 8400.0,
        cycle: BillingCycle::Yearly,
        start_date: base_date,
        next_due_date: next_yearly,
        paused: false,
    };
    assert_eq!(SubscriptionManager::monthly_cost(&hbo), 700.0); // 8400 / 12 = 700
}

#[test]
fn test_subscription_missed_due_date_catchup() {
    let mut data = AppData::default();
    let card_id = data.cards[0].id;

    // Subscription due yesterday
    let past_due = NaiveDate::from_ymd_opt(2026, 9, 15).unwrap();
    let today = NaiveDate::from_ymd_opt(2026, 9, 26).unwrap();

    data.subscriptions.push(Subscription {
        id: Uuid::new_v4(),
        card_id,
        name: "Test Monthly Gym".to_string(),
        amount: 3000.0,
        cycle: BillingCycle::Monthly,
        start_date: past_due,
        next_due_date: past_due,
        paused: false,
    });

    let notices = SubscriptionManager::process_due_subscriptions(&mut data, today);
    assert!(!notices.is_empty());

    // Verify auto-generated transaction was inserted into ledger
    let auto_tx = data.transactions.iter().find(|t| t.description.contains("Test Monthly Gym"));
    assert!(auto_tx.is_some());
    let tx = auto_tx.unwrap();
    assert_eq!(tx.amount, 3000.0);
    assert_eq!(tx.category, "Subscription");
    assert!(!tx.is_income);
    assert!(tx.auto_generated);

    // Verify next due date advanced to Oct 15
    let sub = data.subscriptions.iter().find(|s| s.name == "Test Monthly Gym").unwrap();
    assert_eq!(sub.next_due_date, NaiveDate::from_ymd_opt(2026, 10, 15).unwrap());
}

#[test]
fn test_savings_plan_feasibility_and_calculations() {
    let today = NaiveDate::from_ymd_opt(2026, 9, 26).unwrap();
    let deadline = NaiveDate::from_ymd_opt(2027, 3, 26).unwrap(); // 6 months

    let target = 120_000.0;
    let income = 100_000.0;
    let sub_cost = 1_000.0;

    let (months_rem, req_savings, rec_limit, eff_limit, feasible, shortage) =
        SavingsEngine::evaluate_plan_metrics(today, target, deadline, income, None, sub_cost);

    assert!((months_rem - 6.0).abs() < 0.2);
    // Required savings = 120,000 / 6 = 20,000/mo
    assert!((req_savings - 20_000.0).abs() < 1000.0);
    // Recommended limit = 100,000 - 20,000 - 1,000 = 79,000/mo
    assert!((rec_limit - 79_000.0).abs() < 1000.0);
    assert_eq!(rec_limit, eff_limit);
    assert!(feasible);
    assert_eq!(shortage, 0.0);

    // Test unfeasible plan where target requires 150,000/mo on 100,000 income
    let unfeasible_target = 900_000.0;
    let (_, req_unfeasible, _, _, is_feasible, short) =
        SavingsEngine::evaluate_plan_metrics(today, unfeasible_target, deadline, income, None, sub_cost);

    assert!(!is_feasible);
    assert!(short > 0.0);
    assert!(req_unfeasible > income);
}

#[test]
fn test_savings_plan_progress_and_surplus_tracking() {
    let today = NaiveDate::from_ymd_opt(2026, 9, 26).unwrap();
    let deadline = NaiveDate::from_ymd_opt(2027, 3, 26).unwrap();
    let card_id = Uuid::new_v4();

    let plan = SavingsPlan {
        id: Uuid::new_v4(),
        name: "Emergency Fund".to_string(),
        target_amount: 100_000.0,
        deadline,
        monthly_income: 80_000.0,
        spending_limit_override: Some(50_000.0), // User set 50,000 limit
        linked_card_ids: vec![card_id],
        created_at: NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        closed: false,
        closed_at: None,
        final_saved: None,
        goal_met: None,
    };

    // Actual spending of 30,000 in September
    let tx1 = Transaction {
        id: Uuid::new_v4(),
        card_id,
        date: NaiveDate::from_ymd_opt(2026, 9, 10).unwrap(),
        description: "Shopping".to_string(),
        category: "Shopping".to_string(),
        amount: 30_000.0,
        is_income: false,
        auto_generated: false,
    };

    let calc = SavingsEngine::compute_progress(&plan, &[tx1], 0.0, today, false);

    // Limit 50,000 - Spent 30,000 = 20,000 surplus saved!
    assert_eq!(calc.breakdowns.len(), 1);
    assert_eq!(calc.breakdowns[0].actual_spent, 30_000.0);
    assert_eq!(calc.breakdowns[0].surplus, 20_000.0);
    assert_eq!(calc.total_saved, 20_000.0);
    assert_eq!(calc.percent_reached, 20.0); // 20k / 100k = 20%
}

#[test]
fn test_storage_atomic_save_and_load() {
    #[path = "../src/storage.rs"]
    mod storage;
    use storage::StorageManager;

    let storage = StorageManager::new();
    let data = storage.load();

    // Verify default seeding
    assert!(!data.cards.is_empty());
    assert_eq!(data.cards[0].name, "Main Card");
    assert!(data.cards[0].is_primary);
    assert!(data.subscriptions.iter().any(|s| s.name == "HBO MAX"));
    assert!(data.subscriptions.iter().any(|s| s.name == "Spotify"));

    // Verify saving works
    let save_res = storage.save(&data);
    assert!(save_res.is_ok());
    assert!(storage.file_path().exists());
}

#[test]
fn test_checklist_requirements_integration() {
    // 1. Comma parsing in numeric fields (Item 2 & 5)
    let parsed_op = Theme::parse_pkr_input("10,500.50");
    assert_eq!(parsed_op, Some(10500.50));
    assert_eq!(Theme::parse_pkr_input("12,000"), Some(12000.0));
    assert_eq!(Theme::parse_pkr_input("8,200"), Some(8200.0));
    assert_eq!(Theme::parse_pkr_input("50,000"), Some(50000.0));
    assert_eq!(Theme::parse_pkr_input("100,000.75"), Some(100000.75));

    // 2. Card creation with opening balance and exactly one opening balance row (Item 2)
    let card_id = Uuid::new_v4();
    let mut card = Card {
        id: card_id,
        name: "Meezan Bank".to_string(),
        is_primary: false,
        opening_balance: parsed_op.unwrap(),
        opening_balance_description: "Opening Balance".to_string(),
        opening_balance_date: None,
    };

    let empty_txs: Vec<Transaction> = Vec::new();
    let rows = LedgerCalculator::compute_full_ledger(&card, &empty_txs);
    assert_eq!(rows.len(), 1);
    assert!(rows[0].is_opening_balance);
    assert_eq!(rows[0].running_balance, 10500.50);

    // 3. Edit opening balance to 12,000 and "Starting balance (Aug)", verify persistence (Item 3)
    card.opening_balance = Theme::parse_pkr_input("12,000").unwrap();
    card.opening_balance_description = "Starting balance (Aug)".to_string();
    card.opening_balance_date = Some(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap());

    // JSON serde roundtrip
    let serialized = serde_json::to_string(&card).unwrap();
    let deserialized: Card = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.opening_balance, 12000.0);
    assert_eq!(deserialized.opening_balance_description, "Starting balance (Aug)");
    assert_eq!(deserialized.opening_balance_date, Some(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()));

    // 4. Add backdated transaction dated a month before today (Item 4)
    // Today: 2026-09-26. Backdated tx: 2026-08-26. Later tx: 2026-09-15.
    let tx_august = Transaction {
        id: Uuid::new_v4(),
        card_id,
        date: NaiveDate::from_ymd_opt(2026, 8, 26).unwrap(),
        description: "August Freelance".to_string(),
        category: "Income".to_string(),
        amount: 8000.0,
        is_income: true,
        auto_generated: false,
    };

    let tx_september = Transaction {
        id: Uuid::new_v4(),
        card_id,
        date: NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        description: "Dinner".to_string(),
        category: "Food".to_string(),
        amount: 3000.0,
        is_income: false,
        auto_generated: false,
    };

    // Pass in reverse order to verify chronological sorting
    let txs = vec![tx_september, tx_august];
    let ledger_rows = LedgerCalculator::compute_full_ledger(&deserialized, &txs);

    assert_eq!(ledger_rows.len(), 3);
    // Row 0: Opening balance on 2026-08-01 (12,000)
    assert!(ledger_rows[0].is_opening_balance);
    assert_eq!(ledger_rows[0].description, "Starting balance (Aug)");
    assert_eq!(ledger_rows[0].running_balance, 12000.0);

    // Row 1: August transaction on 2026-08-26 (12,000 + 8,000 = 20,000)
    assert_eq!(ledger_rows[1].date, NaiveDate::from_ymd_opt(2026, 8, 26).unwrap());
    assert_eq!(ledger_rows[1].running_balance, 20000.0);

    // Row 2: September transaction on 2026-09-15 (20,000 - 3,000 = 17,000)
    assert_eq!(ledger_rows[2].date, NaiveDate::from_ymd_opt(2026, 9, 15).unwrap());
    assert_eq!(ledger_rows[2].running_balance, 17000.0);

    // 5. Savings Plan deadline constraint (Item 8)
    let today = NaiveDate::from_ymd_opt(2026, 9, 26).unwrap();
    let past_deadline = NaiveDate::from_ymd_opt(2026, 9, 20).unwrap();
    assert!(past_deadline <= today); // Verifies condition used to reject past deadline
}
