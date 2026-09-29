#![allow(unused, dead_code)]

use chrono::{Datelike, Duration, Local, NaiveDate, Utc};
use egui::{Color32, Context};
use uuid::Uuid;

use savings_tracker::ledger::LedgerCalculator;
use savings_tracker::models::{AppData, BillingCycle, Card, SavingsPlan, Subscription, Transaction};
use savings_tracker::savings::SavingsEngine;
use savings_tracker::subscriptions::SubscriptionManager;
use savings_tracker::ui::accounts::{render_accounts_view, AccountsViewState, SpreadsheetCol};
use savings_tracker::ui::overview::{render_overview_view, OverviewViewState};
use savings_tracker::ui::savings_view::{render_savings_view, SavingsViewState};
use savings_tracker::ui::subscriptions_view::{render_subscriptions_view, SubscriptionsViewState};
use savings_tracker::ui::theme::{self, Theme};


#[test]
fn test_category_color_palette_consistency() {
    // 1. Fixed palette hues from Section 1.1
    let inc = Theme::category_color("Income");
    let food = Theme::category_color("Food");
    let sub = Theme::category_color("Subscription");
    let shop = Theme::category_color("Shopping");
    let trans = Theme::category_color("Transport");
    let other = Theme::category_color("Other");

    assert_eq!(inc, Color32::from_rgb(0x6C, 0x7C, 0xF0));
    assert_eq!(food, Color32::from_rgb(0x3F, 0xCD, 0xA8));
    assert_eq!(sub, Color32::from_rgb(0xF2, 0xA6, 0x3F));
    assert_eq!(shop, Color32::from_rgb(0xE3, 0x6B, 0xB3));
    assert_eq!(trans, Color32::from_rgb(0x5F, 0xB8, 0xE0));
    assert_eq!(other, Color32::from_rgb(0x9C, 0x8C, 0xF0));

    // Case insensitivity & trimming
    assert_eq!(Theme::category_color("  food  "), food);
    assert_eq!(Theme::category_color("INCOME"), inc);
    assert_eq!(Theme::category_color("Subscriptions"), sub);
    assert_eq!(Theme::category_color("transportation"), trans);

    // Module-level function matches Theme method
    assert_eq!(theme::category_color("Food"), food);
    assert_eq!(theme::category_color("Shopping"), shop);

    // Custom categories map deterministically to one of the palette colors
    let custom1 = Theme::category_color("Utilities");
    let custom2 = Theme::category_color("Entertainment");
    let palette = [
        Theme::CAT_INCOME,
        Theme::CAT_FOOD,
        Theme::CAT_SUBSCRIPTION,
        Theme::CAT_SHOPPING,
        Theme::CAT_TRANSPORT,
        Theme::CAT_OTHER,
    ];
    assert!(palette.contains(&custom1));
    assert!(palette.contains(&custom2));
}

#[test]
fn test_savings_plan_status_and_projected_pace() {
    let today = NaiveDate::from_ymd_opt(2026, 9, 29).unwrap();
    let card_id = Uuid::new_v4();

    // Plan with 100,000 target over 10 months (10,000/mo)
    let plan = SavingsPlan {
        id: Uuid::new_v4(),
        name: "Car Downpayment".to_string(),
        target_amount: 100_000.0,
        deadline: NaiveDate::from_ymd_opt(2027, 7, 29).unwrap(),
        monthly_income: 80_000.0,
        spending_limit_override: Some(50_000.0),
        linked_card_ids: vec![card_id],
        created_at: NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        closed: false,
        closed_at: None,
        final_saved: None,
        goal_met: None,
        deduct_overspending: false,
        updated_at: Utc::now(),
        deleted_at: None,
    };

    // Scenario A: Behind pace (spent 65,000 on 50,000 limit = overspent)
    let tx_heavy = Transaction {
        id: Uuid::new_v4(),
        card_id,
        date: NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        description: "Heavy spending".to_string(),
        category: "Shopping".to_string(),
        amount: 65_000.0,
        is_income: false,
        auto_generated: false,
        updated_at: Utc::now(),
        deleted_at: None,
    };

    let calc_behind = SavingsEngine::compute_progress(&plan, &[tx_heavy], 0.0, today, false);
    assert_eq!(calc_behind.total_saved, 0.0);

    // Status evaluation
    let total_days = (plan.deadline - plan.created_at).num_days().max(1) as f64;
    let elapsed_days = (today - plan.created_at).num_days().clamp(0, total_days as i64) as f64;
    let expected_saved = plan.target_amount * (elapsed_days / total_days);
    let diff_behind = calc_behind.total_saved - expected_saved;
    let tolerance = (plan.target_amount * 0.05).max(1000.0);

    let status_behind = if calc_behind.total_saved >= plan.target_amount || diff_behind > tolerance {
        "Ahead"
    } else if diff_behind < -tolerance || !calc_behind.is_feasible {
        "Behind"
    } else {
        "On track"
    };
    assert_eq!(status_behind, "Behind");

    // Scenario B: Ahead pace (spent only 20,000 on 50,000 limit = 30,000 surplus in 1st month vs ~10,000 expected)
    let tx_light = Transaction {
        id: Uuid::new_v4(),
        card_id,
        date: NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        description: "Light spending".to_string(),
        category: "Food".to_string(),
        amount: 20_000.0,
        is_income: false,
        auto_generated: false,
        updated_at: Utc::now(),
        deleted_at: None,
    };

    let calc_ahead = SavingsEngine::compute_progress(&plan, &[tx_light], 0.0, today, false);
    assert_eq!(calc_ahead.total_saved, 30_000.0);

    let diff_ahead = calc_ahead.total_saved - expected_saved;
    let status_ahead = if calc_ahead.total_saved >= plan.target_amount || diff_ahead > tolerance {
        "Ahead"
    } else if diff_ahead < -tolerance || !calc_ahead.is_feasible {
        "Behind"
    } else {
        "On track"
    };
    assert_eq!(status_ahead, "Ahead");

    // Projected pace calculation:
    // Remaining target = 100,000 - 30,000 = 70,000
    // Avg surplus = 30,000/mo
    // Months needed = ceil(70,000 / 30,000) = 3 months
    let rem_target = plan.target_amount - calc_ahead.total_saved;
    let avg_surplus = 30_000.0;
    let months_needed = (rem_target / avg_surplus).ceil() as u32;
    assert_eq!(months_needed, 3);
    let projected_date = today.checked_add_months(chrono::Months::new(months_needed)).unwrap();
    // 3 months from Sep 2026 is Dec 2026, which is WAY ahead of stated deadline of Jul 2027!
    assert!(projected_date < plan.deadline);
}

#[test]
fn test_dashboard_and_12m_subscription_trend() {
    let mut data = AppData::default();
    let card_id = data.cards[0].id;
    let today = NaiveDate::from_ymd_opt(2026, 9, 29).unwrap();

    // Add subscription starting 6 months ago
    let start_date = today.checked_sub_months(chrono::Months::new(6)).unwrap();
    data.subscriptions.push(Subscription {
        id: Uuid::new_v4(),
        card_id,
        name: "Netflix Premium".to_string(),
        amount: 1500.0,
        cycle: BillingCycle::Monthly,
        start_date,
        next_due_date: today + Duration::days(5),
        paused: false,
        updated_at: Utc::now(),
        deleted_at: None,
    });

    // Verify trailing 12 months calculation
    let mut monthly_costs = Vec::with_capacity(12);
    for i in 0..12 {
        let m_date = today.checked_sub_months(chrono::Months::new(11 - i)).unwrap_or(today);
        let m_start = NaiveDate::from_ymd_opt(m_date.year(), m_date.month(), 1).unwrap();
        let next_m_date = m_start.checked_add_months(chrono::Months::new(1)).unwrap_or(m_start);
        let m_end = next_m_date - Duration::days(1);

        let mut cost = 0.0;
        for s in data.subscriptions.iter().filter(|s| s.deleted_at.is_none()) {
            if s.start_date <= m_end {
                cost += SubscriptionManager::monthly_cost(s);
            }
        }
        monthly_costs.push(cost);
    }

    assert_eq!(monthly_costs.len(), 12);
    // Recent month cost includes Netflix 1500.0
    assert!(monthly_costs[11] >= 1500.0);
}

#[test]
fn test_ui_rendering_all_views_at_multiple_window_sizes() {
    let ctx = Context::default();
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

    let mut data = AppData::default();
    let card_id = data.cards[0].id;

    // Add sample transactions for income, food, shopping
    data.transactions.push(Transaction {
        id: Uuid::new_v4(),
        card_id,
        date: NaiveDate::from_ymd_opt(2026, 9, 5).unwrap(),
        description: "Monthly Salary".to_string(),
        category: "Income".to_string(),
        amount: 150_000.0,
        is_income: true,
        auto_generated: false,
        updated_at: Utc::now(),
        deleted_at: None,
    });
    data.transactions.push(Transaction {
        id: Uuid::new_v4(),
        card_id,
        date: NaiveDate::from_ymd_opt(2026, 9, 10).unwrap(),
        description: "Supermarket Groceries".to_string(),
        category: "Food".to_string(),
        amount: 12_500.0,
        is_income: false,
        auto_generated: false,
        updated_at: Utc::now(),
        deleted_at: None,
    });
    data.transactions.push(Transaction {
        id: Uuid::new_v4(),
        card_id,
        date: NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        description: "Electronics Store".to_string(),
        category: "Shopping".to_string(),
        amount: 25_000.0,
        is_income: false,
        auto_generated: false,
        updated_at: Utc::now(),
        deleted_at: None,
    });

    // Add sample savings plan with expanded breakdown
    let plan_id = Uuid::new_v4();
    data.plans.push(SavingsPlan {
        id: plan_id,
        name: "Emergency Fund".to_string(),
        target_amount: 120_000.0,
        deadline: NaiveDate::from_ymd_opt(2027, 3, 31).unwrap(),
        monthly_income: 150_000.0,
        spending_limit_override: Some(60_000.0),
        linked_card_ids: vec![card_id],
        created_at: NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        closed: false,
        closed_at: None,
        final_saved: None,
        goal_met: None,
        deduct_overspending: false,
        updated_at: Utc::now(),
        deleted_at: None,
    });

    let mut overview_st = OverviewViewState::default();
    let mut accounts_st = AccountsViewState {
        selected_card_id: Some(card_id),
        ..Default::default()
    };
    let mut subs_st = SubscriptionsViewState::default();
    let mut savings_st = SavingsViewState::default();
    savings_st.expanded_breakdowns.insert(plan_id); // Expand to render chart and mini-table

    // Test three window sizes per Section 11 & 12:
    // 1. Compact: 800 x 600
    // 2. Laptop: 1200 x 800
    // 3. Maximized: 1600 x 1000
    let test_viewports = [
        egui::vec2(800.0, 600.0),
        egui::vec2(1200.0, 800.0),
        egui::vec2(1600.0, 1000.0),
    ];

    for vp in test_viewports {
        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), vp));

        ctx.run(raw_input, |ctx| {
            // A. Overview View
            egui::CentralPanel::default().show(ctx, |ui| {
                render_overview_view(ui, &data, &mut overview_st);
            });

            // B. Bank Accounts View (Spreadsheet)
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut data_changed = false;
                render_accounts_view(ui, &mut data, &mut accounts_st, &mut data_changed);
            });

            // C. Subscriptions View (with 12m trend chart)
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut data_changed = false;
                render_subscriptions_view(ui, &mut data, &mut subs_st, &mut data_changed);
            });

            // D. Savings Plans View (with trajectory chart and mini-table)
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut data_changed = false;
                render_savings_view(ui, &mut data, &mut savings_st, &mut data_changed);
            });
        });
    }
}
