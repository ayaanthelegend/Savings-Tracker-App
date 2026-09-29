pub mod accounts;
pub mod savings_view;
pub mod subscriptions_view;
pub mod theme;

use crate::ledger::LedgerCalculator;
use crate::models::AppData;
use crate::savings::SavingsEngine;
use crate::storage::StorageManager;
use crate::subscriptions::SubscriptionManager;
use crate::ui::accounts::{render_accounts_view, AccountsViewState};
use crate::ui::savings_view::{render_savings_view, SavingsViewState};
use crate::ui::subscriptions_view::{render_subscriptions_view, SubscriptionsViewState};
use crate::ui::theme::Theme;
use chrono::Local;
use eframe::egui;
use egui::{Align, Color32, Layout, RichText, Rounding, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Accounts,
    Subscriptions,
    SavingsPlans,
}

impl Default for AppTab {
    fn default() -> Self {
        Self::Accounts
    }
}

pub struct SavingsTrackerApp {
    storage: StorageManager,
    data: AppData,
    active_tab: AppTab,

    accounts_state: AccountsViewState,
    subscriptions_state: SubscriptionsViewState,
    savings_state: SavingsViewState,

    system_notices: Vec<String>,
}

impl SavingsTrackerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let storage = StorageManager::new();
        let mut data = storage.load();
        let today = Local::now().date_naive();

        // 1. Process any subscription due dates missed while the app was closed
        let sub_notices = SubscriptionManager::process_due_subscriptions(&mut data, today);

        // 2. Check any savings plans that reached their deadline while the app was closed
        SavingsEngine::check_expirations(&mut data, today);

        // Silent autosave for catchup changes
        let _ = storage.save(&data);

        let initial_card = data.cards.first().map(|c| c.id);

        Self {
            storage,
            data,
            active_tab: AppTab::Accounts,
            accounts_state: AccountsViewState {
                selected_card_id: initial_card,
                ..Default::default()
            },
            subscriptions_state: SubscriptionsViewState::default(),
            savings_state: SavingsViewState::default(),
            system_notices: sub_notices,
        }
    }
}

impl eframe::App for SavingsTrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let today = Local::now().date_naive();
        let mut data_changed = false;

        // --- SIDEBAR: width:196px; background:var(--panel); border-right:1px solid var(--border); padding:18px 12px; ---
        egui::SidePanel::left("left_sidebar")
            .exact_width(196.0)
            .resizable(false)
            .frame(
                egui::Frame::none()
                    .fill(Theme::PANEL)
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER)),
            )
            .show(ctx, |ui| {
                ui.add_space(18.0);

                // Brand: font-family:var(--head); font-weight:700; font-size:16px; dot 9px
                ui.horizontal(|ui| {
                    ui.add_space(12.0);

                    // 9px accent circle dot
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(9.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 4.5, Theme::ACCENT);

                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Savings Tracker")
                            .font(Theme::font_head(16.0))
                            .strong()
                            .color(Theme::TEXT),
                    );
                });

                ui.add_space(20.0);

                // Nav buttons: padding:9px 10px; border-radius:7px; font-size:13.5px;
                let nav_items = [
                    (AppTab::Accounts, "Bank Accounts"),
                    (AppTab::Subscriptions, "Subscriptions"),
                    (AppTab::SavingsPlans, "Savings Plans"),
                ];

                ui.vertical(|ui| {
                    for (tab, label) in nav_items {
                        let is_active = self.active_tab == tab;
                        let item_width = ui.available_width() - 24.0;
                        let item_height = 36.0;

                        ui.horizontal(|ui| {
                            ui.add_space(12.0);

                            let (rect, resp) = ui.allocate_exact_size(
                                Vec2::new(item_width, item_height),
                                egui::Sense::click(),
                            );

                            let bg_color = if is_active {
                                Theme::ACCENT_DIM
                            } else if resp.hovered() {
                                Theme::PANEL2
                            } else {
                                Color32::TRANSPARENT
                            };

                            let text_color = if is_active {
                                Color32::WHITE
                            } else {
                                Theme::MUTED
                            };

                            ui.painter().rect_filled(
                                rect,
                                Rounding::same(Theme::RADIUS_BTN),
                                bg_color,
                            );

                            let text_pos = rect.left_center() + Vec2::new(10.0, 0.0);
                            ui.painter().text(
                                text_pos,
                                egui::Align2::LEFT_CENTER,
                                label,
                                Theme::font_sans(13.5),
                                text_color,
                            );

                            if resp.clicked() {
                                self.active_tab = tab;
                            }
                        });

                        ui.add_space(3.0);
                    }
                });
            });

        // --- TOPBAR: height:52px; border-bottom:1px solid var(--border); padding:0 22px; ---
        egui::TopBottomPanel::top("top_bar")
            .exact_height(52.0)
            .frame(
                egui::Frame::none()
                    .fill(Theme::BG)
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(22.0);

                    // Section title: font-family:var(--head); font-size:15.5px; font-weight:600;
                    let section_title = match self.active_tab {
                        AppTab::Accounts => "Bank Accounts",
                        AppTab::Subscriptions => "Subscriptions",
                        AppTab::SavingsPlans => "Savings Plans",
                    };
                    ui.label(
                        RichText::new(section_title)
                            .font(Theme::font_head(15.5))
                            .strong()
                            .color(Theme::TEXT),
                    );

                    // Right items: Net worth and date with safe 40px margin from window border
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(40.0);

                        // Live date: e.g. Saturday, September 26, 2026
                        ui.label(
                            RichText::new(today.format("%A, %B %d, %Y").to_string())
                                .font(Theme::font_sans(13.0))
                                .color(Theme::MUTED),
                        );

                        ui.add_space(20.0);

                        // Net worth: Net worth <span class="networth">Rs 106,736</span>
                        let mut total_net_worth = 0.0;
                        for card in &self.data.cards {
                            let rows = LedgerCalculator::compute_full_ledger(card, &self.data.transactions);
                            total_net_worth += rows.last().map(|r| r.running_balance).unwrap_or(card.opening_balance);
                        }

                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Net worth ")
                                    .font(Theme::font_sans(13.0))
                                    .color(Theme::MUTED),
                            );
                            ui.label(
                                RichText::new(Theme::format_pkr_whole(total_net_worth))
                                    .font(Theme::font_mono(13.5))
                                    .color(Theme::IN),
                            );
                        });
                    });
                });
            });

        // --- MAIN CONTENT AREA ---
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Theme::BG))
            .show(ctx, |ui| {
                if !self.system_notices.is_empty() {
                    ui.add_space(8.0);
                    egui::Frame::none()
                        .fill(Theme::PANEL)
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER))
                        .rounding(Rounding::same(Theme::RADIUS_BTN))
                        .inner_margin(egui::Margin::symmetric(14.0, 8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("[Notice] Automated:").strong().color(Theme::ACCENT));
                                ui.vertical(|ui| {
                                    for notice in &self.system_notices {
                                        ui.label(RichText::new(notice).small().color(Theme::TEXT));
                                    }
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button("Dismiss").clicked() {
                                        self.system_notices.clear();
                                    }
                                });
                            });
                        });
                    ui.add_space(8.0);
                }

                match self.active_tab {
                    AppTab::Accounts => {
                        render_accounts_view(
                            ui,
                            &mut self.data,
                            &mut self.accounts_state,
                            &mut data_changed,
                        );
                    }
                    AppTab::Subscriptions => {
                        render_subscriptions_view(
                            ui,
                            &mut self.data,
                            &mut self.subscriptions_state,
                            &mut data_changed,
                        );
                    }
                    AppTab::SavingsPlans => {
                        render_savings_view(
                            ui,
                            &mut self.data,
                            &mut self.savings_state,
                            &mut data_changed,
                        );
                    }
                }
            });

        // Silent autosave on any state mutation
        if data_changed {
            if let Err(e) = self.storage.save(&self.data) {
                eprintln!("Failed to save data: {}", e);
            }
        }
    }
}
