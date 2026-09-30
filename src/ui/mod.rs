pub mod accounts;
pub mod login;
pub mod overview;
pub mod savings_view;
pub mod subscriptions_view;
pub mod theme;

use crate::auth::AuthManager;
use crate::ledger::LedgerCalculator;
use crate::models::AppData;
use crate::savings::SavingsEngine;
use crate::storage::StorageManager;
use crate::subscriptions::SubscriptionManager;
use crate::sync::{SyncEvent, SyncManager, SyncStatus};
use crate::ui::accounts::{render_accounts_view, AccountsViewState};
use crate::ui::login::{render_login_view, LoginAction, LoginViewState};
use crate::ui::overview::{render_overview_view, OverviewViewState};
use crate::ui::savings_view::{render_savings_view, SavingsViewState};
use crate::ui::subscriptions_view::{render_subscriptions_view, SubscriptionsViewState};
use crate::ui::theme::Theme;
use chrono::Local;
use eframe::egui;
use egui::{pos2, vec2, Align, Color32, Layout, RichText, Rounding, Stroke, Vec2};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Overview,
    Accounts,
    Subscriptions,
    SavingsPlans,
}

impl Default for AppTab {
    fn default() -> Self {
        Self::Overview
    }
}

pub struct SavingsTrackerApp {
    storage: StorageManager,
    data: AppData,
    active_tab: AppTab,

    overview_state: OverviewViewState,
    accounts_state: AccountsViewState,
    subscriptions_state: SubscriptionsViewState,
    savings_state: SavingsViewState,

    system_notices: Vec<String>,

    // Cloud Sync & Auth
    sync_manager: SyncManager,
    app_data: Arc<Mutex<AppData>>,
    is_logged_in: bool,
    user_email: String,
    login_state: LoginViewState,
    was_focused: bool,
    last_pull_instant: Instant,
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

        let initial_card = data.cards.iter().find(|c| c.deleted_at.is_none()).map(|c| c.id);

        let session = AuthManager::load_session();
        let is_logged_in = session.is_some();
        let user_email = session.as_ref().map(|s| s.user_email.clone()).unwrap_or_default();

        let app_data_arc = Arc::new(Mutex::new(data.clone()));
        let sync_manager = SyncManager::new(Arc::clone(&app_data_arc));

        if is_logged_in {
            sync_manager.check_initial_sync();
        }

        Self {
            storage,
            data,
            app_data: app_data_arc,
            active_tab: AppTab::Overview,
            overview_state: OverviewViewState::default(),
            accounts_state: AccountsViewState {
                selected_card_id: initial_card,
                ..Default::default()
            },
            subscriptions_state: SubscriptionsViewState::default(),
            savings_state: SavingsViewState::default(),
            system_notices: sub_notices,

            sync_manager,
            is_logged_in,
            user_email,
            login_state: LoginViewState::default(),
            was_focused: true,
            last_pull_instant: Instant::now(),
        }
    }
}

impl eframe::App for SavingsTrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- 1. PROCESS SYNC EVENTS ---
        while let Some(event) = self.sync_manager.try_recv_event() {
            match event {
                SyncEvent::DataMerged(new_data) => {
                    self.data = *new_data;
                    *self.app_data.lock().unwrap() = self.data.clone();
                    let _ = self.storage.save(&self.data);
                    ctx.request_repaint();
                }
                SyncEvent::PushCompleted => {
                    ctx.request_repaint();
                }
                SyncEvent::StatusChanged(_) => {
                    ctx.request_repaint();
                }
                SyncEvent::SessionRefreshed(new_session) => {
                    self.user_email = new_session.user_email;
                    ctx.request_repaint();
                }
                SyncEvent::AuthFailed(msg) => {
                    self.is_logged_in = false;
                    self.login_state.error_message = Some(format!("Session expired: {}", msg));
                    ctx.request_repaint();
                }
            }
        }

        // --- 2. FOCUS DETECTION & PERIODIC PULL (Section 3) ---
        let is_focused = ctx.input(|i| i.viewport().focused).unwrap_or(true);
        if self.is_logged_in {
            if is_focused && !self.was_focused {
                // Just regained focus: immediate pull
                self.sync_manager.trigger_pull();
                self.last_pull_instant = Instant::now();
            } else if self.last_pull_instant.elapsed() >= Duration::from_secs(300) {
                // Periodic 5-minute pull
                self.sync_manager.trigger_pull();
                self.last_pull_instant = Instant::now();
            }
        }
        self.was_focused = is_focused;

        // --- 3. SHOW LOGIN SCREEN IF NOT AUTHENTICATED ---
        if !self.is_logged_in {
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(Theme::BG))
                .show(ctx, |ui| {
                    let action = render_login_view(ui, &mut self.login_state);
                    if let LoginAction::LoggedIn(session) = action {
                        self.is_logged_in = true;
                        self.user_email = session.user_email.clone();
                        self.sync_manager.update_session(session);
                        self.sync_manager.check_initial_sync();
                        ctx.request_repaint();
                    }
                });
            return;
        }

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

                // Brand: font-family:var(--head); font-weight:700; font-size:16px; dot 9px + version tag
                ui.horizontal(|ui| {
                    ui.add_space(10.0);

                    // 9px accent circle dot
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(9.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 4.5, Theme::ACCENT);

                    ui.add_space(7.0);
                    ui.label(
                        RichText::new("Savings Tracker")
                            .font(Theme::font_head(15.0))
                            .strong()
                            .color(Theme::TEXT),
                    );

                    ui.add_space(4.0);
                    // Version tag
                    egui::Frame::none()
                        .fill(Theme::PANEL2)
                        .rounding(Rounding::same(3.0))
                        .inner_margin(egui::Margin::symmetric(4.0, 1.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("v0.1")
                                    .font(Theme::font_sans(9.5))
                                    .color(Theme::MUTED),
                            );
                        });
                });

                ui.add_space(16.0);

                // Nav list: Overview -> Bank Accounts -> Subscriptions -> Savings Plans
                let nav_items = [
                    (AppTab::Overview, "Overview"),
                    (AppTab::Accounts, "Bank Accounts"),
                    (AppTab::Subscriptions, "Subscriptions"),
                    (AppTab::SavingsPlans, "Savings Plans"),
                ];

                let item_width = ui.available_width() - 20.0;
                let item_height = 36.0;

                ui.vertical(|ui| {
                    for (tab, label) in nav_items {
                        let is_active = self.active_tab == tab;

                        ui.horizontal(|ui| {
                            ui.add_space(10.0);

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

                            let icon_color = if is_active {
                                Color32::WHITE
                            } else {
                                Theme::MUTED
                            };

                            ui.painter().rect_filled(
                                rect,
                                Rounding::same(Theme::RADIUS_BTN),
                                bg_color,
                            );

                            // 15px icon + 9px gap + label
                            let icon_center = pos2(rect.left() + 18.0, rect.center().y);
                            paint_nav_icon(ui, icon_center, tab, icon_color);

                            let text_pos = rect.left_center() + Vec2::new(32.0, 0.0);
                            ui.painter().text(
                                text_pos,
                                egui::Align2::LEFT_CENTER,
                                label,
                                Theme::font_sans(13.0),
                                text_color,
                            );

                            if resp.clicked() {
                                self.active_tab = tab;
                            }
                        });

                        ui.add_space(3.0);
                    }
                });

                // Bottom section: User Info, Sync Status Badge, Sync Trigger, Logout
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.add_space(14.0);

                    // Logout Button (Section 2.2)
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        if ui.add_sized(
                            Vec2::new(item_width, 28.0),
                            egui::Button::new(
                                RichText::new("Log out")
                                    .font(Theme::font_sans(12.0))
                                    .color(Theme::MUTED),
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0_f32, Theme::BORDER)),
                        ).clicked() {
                            let _ = AuthManager::clear_session();
                            self.sync_manager.logout();
                            self.is_logged_in = false;
                            self.login_state.password.clear();
                        }
                    });

                    ui.add_space(8.0);

                    // Sync Status Indicator
                    let status = self.sync_manager.current_status();
                    let (status_color, status_text) = match &status {
                        SyncStatus::Idle => (Theme::FAINT, "Cloud sync idle".to_string()),
                        SyncStatus::Syncing => (Theme::ACCENT, "Syncing...".to_string()),
                        SyncStatus::Synced(time) => {
                            let local_time: chrono::DateTime<chrono::Local> = chrono::DateTime::from(*time);
                            (Theme::IN, format!("Synced {}", local_time.format("%H:%M")))
                        }
                        SyncStatus::Offline(_err) => (Theme::EXPENSE, "Offline (queued)".to_string()),
                        SyncStatus::AuthError(_) => (Theme::EXPENSE, "Auth error".to_string()),
                    };

                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 3.5, status_color);
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(status_text)
                                .font(Theme::font_sans(11.0))
                                .color(Theme::MUTED),
                        );

                        // Manual Sync Button
                        if ui.link(RichText::new("↻").font(Theme::font_head(13.0)).color(Theme::ACCENT)).clicked() {
                            self.sync_manager.trigger_pull();
                        }
                    });

                    ui.add_space(6.0);

                    // User Email
                    if !self.user_email.is_empty() {
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(&self.user_email)
                                    .font(Theme::font_sans(11.0))
                                    .color(Theme::FAINT),
                            );
                        });
                    }

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        ui.painter().hline(
                            10.0..=(10.0 + item_width),
                            ui.cursor().top(),
                            Stroke::new(1.0_f32, Theme::BORDER),
                        );
                    });
                });
            });

        // --- TOPBAR: height:52px; border-bottom:1px solid var(--border); padding:0 24px; ---
        egui::TopBottomPanel::top("top_bar")
            .exact_height(52.0)
            .frame(
                egui::Frame::none()
                    .fill(Theme::BG)
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(Theme::PAGE_MARGIN);

                    // Section title: font-family:var(--head); font-size:15.5px; font-weight:600;
                    let section_title = match self.active_tab {
                        AppTab::Overview => "Overview",
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

                    // Right items: Net worth (hero-style mono figure, --in) -> 22px gap -> date -> 24px right padding
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(Theme::PAGE_MARGIN);

                        // Live date: e.g. Saturday, September 26, 2026
                        ui.label(
                            RichText::new(today.format("%A, %B %d, %Y").to_string())
                                .font(Theme::font_sans(13.0))
                                .color(Theme::MUTED),
                        );

                        ui.add_space(22.0);

                        // Net worth: omit deleted cards
                        let mut total_net_worth = 0.0;
                        for card in self.data.cards.iter().filter(|c| c.deleted_at.is_none()) {
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
                // Subtle grid background pattern
                Theme::paint_grid_background(ui, ui.max_rect());

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
                    AppTab::Overview => {
                        render_overview_view(
                            ui,
                            &self.data,
                            &mut self.overview_state,
                        );
                    }
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

        // --- ATOMIC LOCAL DISK SAVE & CLOUD SYNC QUEUE ON MUTATION (Section 2.3) ---
        if data_changed {
            if let Err(e) = self.storage.save(&self.data) {
                eprintln!("Failed to save data locally: {}", e);
            }
            *self.app_data.lock().unwrap() = self.data.clone();
            self.sync_manager.enqueue_push();
        }
    }
}

fn paint_nav_icon(ui: &mut egui::Ui, center: egui::Pos2, tab: AppTab, color: Color32) {
    let p = ui.painter();
    let stroke = Stroke::new(1.3_f32, color);
    match tab {
        AppTab::Overview => {
            // 2x2 grid of small rounded squares (15px icon)
            let s = 4.5;
            let gap = 2.0;
            let top_left = center - vec2(s + gap * 0.5, s + gap * 0.5);
            p.rect_filled(egui::Rect::from_min_size(top_left, vec2(s, s)), Rounding::same(1.0), color);
            p.rect_filled(egui::Rect::from_min_size(top_left + vec2(s + gap, 0.0), vec2(s, s)), Rounding::same(1.0), color);
            p.rect_filled(egui::Rect::from_min_size(top_left + vec2(0.0, s + gap), vec2(s, s)), Rounding::same(1.0), color);
            p.rect_filled(egui::Rect::from_min_size(top_left + vec2(s + gap, s + gap), vec2(s, s)), Rounding::same(1.0), color);
        }
        AppTab::Accounts => {
            // Bank card outline
            let r = egui::Rect::from_center_size(center, vec2(13.0, 9.5));
            p.rect_stroke(r, Rounding::same(1.5), stroke);
            p.line_segment([pos2(r.left(), r.top() + 3.0), pos2(r.right(), r.top() + 3.0)], stroke);
        }
        AppTab::Subscriptions => {
            // Cyclic reload / cycle circle
            let radius = 5.0;
            p.circle_stroke(center, radius, stroke);
            p.line_segment([center + vec2(radius - 1.5, -2.5), center + vec2(radius + 1.0, 0.0)], stroke);
        }
        AppTab::SavingsPlans => {
            // Upward trend line
            p.line_segment([center + vec2(-5.5, 3.5), center + vec2(-1.5, -0.5)], stroke);
            p.line_segment([center + vec2(-1.5, -0.5), center + vec2(1.5, 2.0)], stroke);
            p.line_segment([center + vec2(1.5, 2.0), center + vec2(5.5, -4.0)], stroke);
            p.line_segment([center + vec2(2.5, -4.0), center + vec2(5.5, -4.0)], stroke);
            p.line_segment([center + vec2(5.5, -1.0), center + vec2(5.5, -4.0)], stroke);
        }
    }
}
