use crate::models::{AppData, SavingsPlan};
use crate::savings::SavingsEngine;
use crate::subscriptions::SubscriptionManager;
use crate::ui::theme::Theme;
use chrono::{Datelike, Local, NaiveDate};
use egui::{vec2, Align, Color32, Layout, RichText, Rounding, Stroke, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use egui_plot::{Corner, Legend, Line, Plot, PlotPoints};
use uuid::Uuid;

#[derive(Default)]
pub struct SavingsViewState {
    pub active_subtab: SavingsSubTab,
    pub show_create_modal: bool,
    pub editing_plan_id: Option<Uuid>,

    // Plan input fields
    pub name: String,
    pub target_amount: String,
    pub deadline_date: String,
    pub monthly_income: String,
    pub override_spending_limit: bool,
    pub spending_limit_override_val: String,
    pub link_all_cards: bool,
    pub selected_card_id: Option<Uuid>,
    pub modal_error: Option<String>,

    pub plan_to_delete: Option<Uuid>,
    pub expanded_breakdowns: std::collections::HashSet<Uuid>,
    pub deduct_overspending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavingsSubTab {
    Active,
    History,
}

impl Default for SavingsSubTab {
    fn default() -> Self {
        Self::Active
    }
}

impl SavingsViewState {
    pub fn open_create(&mut self, default_card: Option<Uuid>) {
        self.show_create_modal = true;
        self.editing_plan_id = None;
        self.name.clear();
        self.target_amount.clear();
        let def_deadline = Local::now().date_naive()
            .checked_add_months(chrono::Months::new(6))
            .unwrap_or_else(|| Local::now().date_naive() + chrono::Duration::days(180));
        self.deadline_date = Theme::format_input_date(&def_deadline);
        self.monthly_income.clear();
        self.override_spending_limit = false;
        self.spending_limit_override_val.clear();
        self.link_all_cards = true;
        self.selected_card_id = default_card;
        self.modal_error = None;
    }

    pub fn open_edit(&mut self, plan: &SavingsPlan) {
        self.show_create_modal = true;
        self.editing_plan_id = Some(plan.id);
        self.name = plan.name.clone();
        self.target_amount = format!("{:.0}", plan.target_amount);
        self.deadline_date = Theme::format_input_date(&plan.deadline);
        self.monthly_income = format!("{:.0}", plan.monthly_income);
        if let Some(ov) = plan.spending_limit_override {
            self.override_spending_limit = true;
            self.spending_limit_override_val = format!("{:.0}", ov);
        } else {
            self.override_spending_limit = false;
            self.spending_limit_override_val.clear();
        }
        self.link_all_cards = plan.linked_card_ids.is_empty();
        self.selected_card_id = plan.linked_card_ids.first().cloned();
        self.modal_error = None;
    }
}

pub fn render_savings_view(
    ui: &mut Ui,
    data: &mut AppData,
    state: &mut SavingsViewState,
    data_changed: &mut bool,
) {
    let today = Local::now().date_naive();
    SavingsEngine::check_expirations(data, today);

    let avail_width = (ui.available_width() - 2.0 * Theme::PAGE_MARGIN).max(0.0);
    let content_width = avail_width.min(Theme::CONTENT_MAX_WIDTH);
    let side_margin = Theme::PAGE_MARGIN + (avail_width - content_width) / 2.0;

    ui.add_space(Theme::CARD_GAP);

    // --- SUB-TABS (Active Plans vs History) ---
    ui.horizontal(|ui| {
        ui.add_space(side_margin);
        ui.vertical(|ui| {
            ui.set_width(content_width);
            ui.set_max_width(content_width);

            ui.horizontal(|ui| {
                let is_active = state.active_subtab == SavingsSubTab::Active;
                let is_hist = state.active_subtab == SavingsSubTab::History;

                let active_pill = egui::Frame::none()
                    .fill(if is_active { Theme::PANEL2 } else { Theme::PANEL })
                    .stroke(if is_active { Theme::stroke_accent_dim() } else { Theme::stroke_border() })
                    .rounding(Rounding::same(Theme::RADIUS_TAB))
                    .inner_margin(egui::Margin::symmetric(14.0, 6.0));

                if active_pill.show(ui, |ui| {
                    ui.label(RichText::new("Active Plans").font(Theme::font_sans(13.0)).color(if is_active { Theme::TEXT } else { Theme::MUTED }));
                }).response.interact(egui::Sense::click()).clicked() {
                    state.active_subtab = SavingsSubTab::Active;
                }

                ui.add_space(8.0);

                let hist_count = data.plans.iter().filter(|p| p.deleted_at.is_none() && p.closed).count();
                let hist_pill = egui::Frame::none()
                    .fill(if is_hist { Theme::PANEL2 } else { Theme::PANEL })
                    .stroke(if is_hist { Theme::stroke_accent_dim() } else { Theme::stroke_border() })
                    .rounding(Rounding::same(Theme::RADIUS_TAB))
                    .inner_margin(egui::Margin::symmetric(14.0, 6.0));

                if hist_pill.show(ui, |ui| {
                    let label = if hist_count > 0 { format!("History ({})", hist_count) } else { "History".to_string() };
                    ui.label(RichText::new(label).font(Theme::font_sans(13.0)).color(if is_hist { Theme::TEXT } else { Theme::MUTED }));
                }).response.interact(egui::Sense::click()).clicked() {
                    state.active_subtab = SavingsSubTab::History;
                }

                ui.add_space(16.0);
                ui.checkbox(&mut state.deduct_overspending, RichText::new("Deduct overspending from surplus").font(Theme::font_sans(12.0)).color(Theme::MUTED));
            });
        });
    });

    ui.add_space(Theme::CARD_GAP);

    match state.active_subtab {
        SavingsSubTab::Active => render_active_plans_grid(ui, data, state, today, content_width, side_margin),
        SavingsSubTab::History => render_history_plans_grid(ui, data, state, content_width, side_margin),
    }

    // Confirmation Modal
    if let Some(del_id) = state.plan_to_delete {
        egui::Window::new("Confirm Delete Savings Plan")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(320.0);
                ui.label("Are you sure you want to delete this savings plan?");
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Delete").color(Theme::OUT).strong()).clicked() {
                        let now = chrono::Utc::now();
                        if let Some(p) = data.plans.iter_mut().find(|p| p.id == del_id) {
                            p.deleted_at = Some(now);
                            p.updated_at = now;
                        }
                        *data_changed = true;
                        state.plan_to_delete = None;
                    }
                    if ui.button("Cancel").clicked() {
                        state.plan_to_delete = None;
                    }
                });
            });
    }

    if state.show_create_modal {
        render_plan_modal(ui, data, state, today, data_changed);
    }
}

fn render_active_plans_grid(
    ui: &mut Ui,
    data: &mut AppData,
    state: &mut SavingsViewState,
    today: NaiveDate,
    content_width: f32,
    side_margin: f32,
) {
    let active_plans: Vec<SavingsPlan> = data.plans.iter().filter(|p| p.deleted_at.is_none() && !p.closed).cloned().collect();

    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(side_margin);
            ui.vertical(|ui| {
                ui.set_width(content_width);
                ui.set_max_width(content_width);

                let mut plan_to_edit = None;

                for plan in &active_plans {
                    let prorated_sub = SubscriptionManager::total_monthly_cost(
                        &data.subscriptions,
                        if plan.linked_card_ids.is_empty() { None } else { Some(&plan.linked_card_ids) },
                    );

                    let calc = SavingsEngine::compute_progress(
                        plan,
                        &data.transactions,
                        prorated_sub,
                        today,
                        state.deduct_overspending,
                    );

                    // Calculate plan status and color per Section 4.3
                    let total_days = (plan.deadline - plan.created_at).num_days().max(1) as f64;
                    let elapsed_days = (today - plan.created_at).num_days().clamp(0, total_days as i64) as f64;
                    let expected_saved = plan.target_amount * (elapsed_days / total_days);
                    let diff = calc.total_saved - expected_saved;
                    let tolerance = (plan.target_amount * 0.05).max(1000.0);

                    let (status_text, status_color) = if calc.total_saved >= plan.target_amount || diff > tolerance {
                        ("Ahead", Theme::STATUS_AHEAD)
                    } else if diff < -tolerance || !calc.is_feasible {
                        ("Behind", Theme::STATUS_BEHIND)
                    } else {
                        ("On track", Theme::STATUS_ON_TRACK)
                    };

                    // Compute projected completion date from recent months' actual surplus
                    let rem_target = (plan.target_amount - calc.total_saved).max(0.0);
                    let recent_surpluses: Vec<f64> = calc.breakdowns.iter().rev().take(3).map(|b| b.surplus).collect();
                    let avg_surplus = if !recent_surpluses.is_empty() {
                        recent_surpluses.iter().sum::<f64>() / recent_surpluses.len() as f64
                    } else {
                        0.0
                    };

                    let projected_str = if rem_target <= 0.0 {
                        "At this pace: Target reached!".to_string()
                    } else if avg_surplus > 10.0 {
                        let months_needed = (rem_target / avg_surplus).ceil() as u32;
                        let projected_date = today.checked_add_months(chrono::Months::new(months_needed))
                            .unwrap_or(today);
                        format!("At this pace: {}", projected_date.format("%b %Y"))
                    } else {
                        "At this pace: Target unachievable at current rate".to_string()
                    };

                    let card_frame = egui::Frame::none()
                        .fill(Theme::PANEL)
                        .stroke(Theme::stroke_border())
                        .rounding(Rounding::same(Theme::RADIUS_CARD))
                        .inner_margin(egui::Margin::symmetric(20.0, Theme::CARD_PADDING));

                    let card_resp = card_frame.show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        // Row 1: Plan Name and Status badge + Deadline date
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&plan.name)
                                    .font(Theme::font_head(15.0))
                                    .strong()
                                    .color(Theme::TEXT),
                            );

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let days_left = (plan.deadline - today).num_days();
                                let deadline_str = if days_left > 0 {
                                    format!("{} ({}d left)", Theme::format_date(&plan.deadline), days_left)
                                } else {
                                    "Deadline passed".to_string()
                                };
                                ui.label(
                                    RichText::new(deadline_str)
                                        .font(Theme::font_sans(11.0))
                                        .color(Theme::MUTED),
                                );

                                ui.add_space(8.0);

                                // Status badge pill
                                egui::Frame::none()
                                    .fill(Color32::from_rgba_premultiplied(
                                        (status_color.r() as f32 * 0.15) as u8,
                                        (status_color.g() as f32 * 0.15) as u8,
                                        (status_color.b() as f32 * 0.15) as u8,
                                        38,
                                    ))
                                    .stroke(Stroke::new(1.0_f32, status_color))
                                    .rounding(Rounding::same(Theme::RADIUS_BADGE))
                                    .inner_margin(egui::Margin::symmetric(7.0, 2.5))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(status_text)
                                                .font(Theme::font_sans(10.5))
                                                .strong()
                                                .color(status_color),
                                        );
                                    });
                            });
                        });

                        ui.add_space(6.0);

                        // Progress Bar matching .bar
                        let progress_frac = (calc.percent_reached / 100.0).clamp(0.0, 1.0) as f32;
                        let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 7.0), egui::Sense::hover());
                        ui.painter().rect_filled(bar_rect, Rounding::same(Theme::RADIUS_BAR), Theme::PANEL2);
                        let fill_w = (bar_rect.width() * progress_frac).clamp(0.0, bar_rect.width());
                        if fill_w > 0.0 {
                            let fill_rect = egui::Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, 7.0));
                            ui.painter().rect_filled(fill_rect, Rounding::same(Theme::RADIUS_BAR), status_color);
                        }

                        ui.add_space(6.0);

                        // Row 2: Stat line — split "Saved Rs X" / "/ Rs Y target" / "Z%"
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("Saved {}", Theme::format_pkr(calc.total_saved)))
                                    .font(Theme::font_sans(12.5))
                                    .color(Theme::TEXT),
                            );
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(format!("/ {} target", Theme::format_pkr_whole(plan.target_amount)))
                                    .font(Theme::font_sans(12.5))
                                    .color(Theme::MUTED),
                            );
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(format!("{:.0}%", calc.percent_reached))
                                    .font(Theme::font_sans(12.5))
                                    .strong()
                                    .color(status_color),
                            );

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if !calc.is_feasible {
                                    ui.label(RichText::new(format!("Shortage: Rs {:.0}/mo", calc.shortage)).font(Theme::font_sans(11.0)).color(Theme::OUT));
                                } else {
                                    ui.label(
                                        RichText::new(format!("Limit: {}/mo", Theme::format_pkr(calc.effective_limit)))
                                            .font(Theme::font_sans(11.0))
                                            .color(Theme::MUTED),
                                    );
                                }
                            });
                        });

                        // 12px gap, then Projected-completion line directly beneath Row 2
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(&projected_str)
                                .font(Theme::font_sans(11.5))
                                .color(Theme::MUTED),
                        );

                        ui.add_space(8.0);

                        // Row 3: Action links (View breakdown / Edit / Delete)
                        let is_expanded = state.expanded_breakdowns.contains(&plan.id);
                        ui.horizontal(|ui| {
                            let expand_label = if is_expanded { "Hide breakdown" } else { "View breakdown" };
                            if ui.link(RichText::new(expand_label).font(Theme::font_sans(12.0)).color(Theme::ACCENT)).clicked() {
                                if is_expanded {
                                    state.expanded_breakdowns.remove(&plan.id);
                                } else {
                                    state.expanded_breakdowns.insert(plan.id);
                                }
                            }

                            ui.add_space(14.0);
                            if ui.link(RichText::new("Edit").font(Theme::font_sans(12.0)).color(Theme::FAINT)).clicked() {
                                plan_to_edit = Some(plan.clone());
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.link(RichText::new("Delete").font(Theme::font_sans(12.0)).color(Theme::FAINT)).clicked() {
                                    state.plan_to_delete = Some(plan.id);
                                }
                            });
                        });

                        // EXPANDED BREAKDOWN: Trajectory Chart (4.1) + Mini-Spreadsheet (4.2)
                        if is_expanded {
                            ui.add_space(16.0);
                            let (sep_r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), egui::Sense::hover());
                            ui.painter().line_segment([sep_r.left_top(), sep_r.right_top()], Theme::stroke_border());
                            ui.add_space(12.0);

                            // Section header
                            ui.label(
                                RichText::new("Savings Trajectory")
                                    .font(Theme::font_head(13.5))
                                    .strong()
                                    .color(Theme::TEXT),
                            );
                            ui.add_space(8.0);

                            // 1. Trajectory Chart
                            let total_plan_months = ((plan.deadline.year() - plan.created_at.year()) * 12
                                + plan.deadline.month() as i32
                                - plan.created_at.month() as i32)
                                .max(1) as f64;

                            let req_pace_points = PlotPoints::new(vec![
                                [0.0, 0.0],
                                [total_plan_months, plan.target_amount],
                            ]);
                            let req_line = Line::new(req_pace_points)
                                .name("Required pace")
                                .color(Theme::ACCENT)
                                .width(1.8_f32);

                            let mut actual_points = vec![[0.0, 0.0]];
                            let mut running_sum = 0.0;
                            for (idx, b) in calc.breakdowns.iter().enumerate() {
                                running_sum += b.surplus;
                                actual_points.push([(idx + 1) as f64, running_sum]);
                            }
                            let act_line = Line::new(PlotPoints::new(actual_points))
                                .name("Actual saved")
                                .color(status_color)
                                .width(2.2_f32);

                            Plot::new(format!("plan_plot_{}", plan.id))
                                .height(165.0)
                                .allow_drag(false)
                                .allow_zoom(false)
                                .allow_scroll(false)
                                .show_grid([true, true])
                                .legend(Legend::default().position(Corner::RightTop))
                                .x_axis_formatter(|mark, _range| format!("M{}", mark.value as i64))
                                .y_axis_formatter(|mark, _range| Theme::format_pkr_whole(mark.value))
                                .show(ui, |plot_ui| {
                                    plot_ui.line(req_line);
                                    plot_ui.line(act_line);
                                });

                            // 16px vertical gap
                            ui.add_space(16.0);

                            // 2. Monthly Breakdown Mini-Spreadsheet
                            ui.label(
                                RichText::new("Monthly Breakdown")
                                    .font(Theme::font_head(13.5))
                                    .strong()
                                    .color(Theme::TEXT),
                            );
                            ui.add_space(8.0);

                            let draw_mini_border = |ui: &Ui, is_first: bool| {
                                let r = ui.max_rect();
                                ui.painter().line_segment([r.right_top(), r.right_bottom()], Theme::stroke_border());
                                ui.painter().line_segment([r.left_bottom(), r.right_bottom()], Theme::stroke_border());
                                if is_first {
                                    ui.painter().line_segment([r.left_top(), r.left_bottom()], Theme::stroke_border());
                                }
                            };

                            TableBuilder::new(ui)
                                .id_salt(format!("mini_table_{}", plan.id))
                                .striped(false)
                                .resizable(false)
                                .cell_layout(Layout::left_to_right(Align::Center))
                                .column(Column::initial(80.0))                  // Month
                                .column(Column::remainder().at_least(65.0))     // Income
                                .column(Column::remainder().at_least(65.0))     // Limit
                                .column(Column::remainder().at_least(65.0))     // Spent
                                .column(Column::remainder().at_least(90.0))     // Surplus/Deficit
                                .column(Column::remainder().at_least(90.0))     // Running total
                                .header(26.0, |mut header| {
                                    header.col(|ui| {
                                        draw_mini_border(ui, true);
                                        ui.add_space(6.0);
                                        ui.label(RichText::new("Month").font(Theme::font_sans(11.0)).color(Theme::MUTED));
                                    });
                                    header.col(|ui| {
                                        draw_mini_border(ui, false);
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            ui.add_space(8.0);
                                            ui.label(RichText::new("Income").font(Theme::font_sans(11.0)).color(Theme::MUTED));
                                        });
                                    });
                                    header.col(|ui| {
                                        draw_mini_border(ui, false);
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            ui.add_space(8.0);
                                            ui.label(RichText::new("Limit").font(Theme::font_sans(11.0)).color(Theme::MUTED));
                                        });
                                    });
                                    header.col(|ui| {
                                        draw_mini_border(ui, false);
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            ui.add_space(8.0);
                                            ui.label(RichText::new("Spent").font(Theme::font_sans(11.0)).color(Theme::MUTED));
                                        });
                                    });
                                    header.col(|ui| {
                                        draw_mini_border(ui, false);
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            ui.add_space(8.0);
                                            ui.label(RichText::new("Surplus").font(Theme::font_sans(11.0)).color(Theme::MUTED));
                                        });
                                    });
                                    header.col(|ui| {
                                        draw_mini_border(ui, false);
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            ui.add_space(8.0);
                                            ui.label(RichText::new("Running total").font(Theme::font_sans(11.0)).color(Theme::MUTED));
                                        });
                                    });
                                })
                                .body(|mut body| {
                                    let mut running_sum = 0.0;
                                    for b in &calc.breakdowns {
                                        running_sum += b.surplus;
                                        body.row(28.0, |mut r| {
                                            r.col(|ui| {
                                                draw_mini_border(ui, true);
                                                ui.add_space(6.0);
                                                ui.label(RichText::new(&b.label).font(Theme::font_sans(11.5)).color(Theme::TEXT));
                                            });
                                            r.col(|ui| {
                                                draw_mini_border(ui, false);
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    ui.add_space(8.0);
                                                    Theme::money_label(ui, &Theme::format_pkr_whole(b.monthly_income), Theme::TEXT, 11.5);
                                                });
                                            });
                                            r.col(|ui| {
                                                draw_mini_border(ui, false);
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    ui.add_space(8.0);
                                                    Theme::money_label(ui, &Theme::format_pkr_whole(b.spending_limit), Theme::MUTED, 11.5);
                                                });
                                            });
                                            r.col(|ui| {
                                                draw_mini_border(ui, false);
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    ui.add_space(8.0);
                                                    Theme::money_label(ui, &Theme::format_pkr_whole(b.actual_spent), Theme::TEXT, 11.5);
                                                });
                                            });
                                            r.col(|ui| {
                                                draw_mini_border(ui, false);
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    ui.add_space(8.0);
                                                    let s_col = if b.surplus >= 0.0 { Theme::IN } else { Theme::OUT };
                                                    let sign = if b.surplus >= 0.0 { "+" } else { "" };
                                                    ui.label(RichText::new(format!("{}{}", sign, Theme::format_pkr_whole(b.surplus))).font(Theme::font_mono(11.5)).color(s_col));
                                                });
                                            });
                                            r.col(|ui| {
                                                draw_mini_border(ui, false);
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    ui.add_space(8.0);
                                                    let r_col = if running_sum >= 0.0 { Theme::IN } else { Theme::OUT };
                                                    Theme::money_label(ui, &Theme::format_pkr_whole(running_sum), r_col, 11.5);
                                                });
                                            });
                                        });
                                    }
                                });
                        }
                    });

                    // 3px top accent bar per Section 1.4 & 4.3
                    Theme::paint_card_accent_bar(ui.painter(), card_resp.response.rect, status_color, Theme::RADIUS_CARD);

                    ui.add_space(Theme::CARD_GAP);
                }

                // --- DASHED "+ New savings plan" FULL-WIDTH CARD AT END OF STACK ---
                let def_card = data.cards.first().map(|c| c.id);
                let (add_rect, add_resp) = ui.allocate_exact_size(
                    Vec2::new(content_width, 54.0),
                    egui::Sense::click(),
                );

                let add_bg = if add_resp.hovered() { Theme::PANEL2 } else { Theme::PANEL };
                ui.painter().rect(
                    add_rect,
                    Rounding::same(Theme::RADIUS_CARD),
                    add_bg,
                    Theme::dashed_stroke(),
                );

                let add_galley = ui.painter().layout_no_wrap(
                    "+ New savings plan".to_string(),
                    Theme::font_sans(14.0),
                    Theme::ACCENT,
                );
                let text_pos = add_rect.center() - add_galley.size() / 2.0;
                ui.painter().galley(text_pos, add_galley, Theme::ACCENT);

                if add_resp.clicked() {
                    state.open_create(def_card);
                }

                if let Some(plan) = plan_to_edit {
                    state.open_edit(&plan);
                }

                ui.add_space(Theme::PAGE_MARGIN);
            });
        });
    });
}

fn render_history_plans_grid(
    ui: &mut Ui,
    data: &mut AppData,
    state: &mut SavingsViewState,
    content_width: f32,
    side_margin: f32,
) {
    let closed_plans: Vec<SavingsPlan> = data.plans.iter().filter(|p| p.deleted_at.is_none() && p.closed).cloned().collect();

    if closed_plans.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(side_margin);
            ui.vertical(|ui| {
                ui.set_width(content_width);
                ui.set_max_width(content_width);
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(RichText::new("No past savings plans in history.").font(Theme::font_sans(14.0)).color(Theme::MUTED));
                    ui.label(RichText::new("Plans reach their deadline and automatically archive here.").font(Theme::font_sans(12.0)).color(Theme::FAINT));
                });
            });
        });
        return;
    }

    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(side_margin);
            ui.vertical(|ui| {
                ui.set_width(content_width);
                ui.set_max_width(content_width);

                for plan in &closed_plans {
                    let card_frame = egui::Frame::none()
                        .fill(Theme::PANEL)
                        .stroke(Theme::stroke_border())
                        .rounding(Rounding::same(Theme::RADIUS_CARD))
                        .inner_margin(egui::Margin::symmetric(20.0, Theme::CARD_PADDING));

                    card_frame.show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        // Row 1: Plan name and goal status
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&plan.name)
                                    .font(Theme::font_head(15.0))
                                    .strong()
                                    .color(Theme::MUTED),
                            );

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if plan.goal_met.unwrap_or(false) {
                                    ui.label(RichText::new("Goal Achieved").font(Theme::font_sans(11.0)).color(Theme::IN));
                                } else {
                                    ui.label(RichText::new("Goal Missed").font(Theme::font_sans(11.0)).color(Theme::FAINT));
                                }
                            });
                        });

                        ui.add_space(6.0);

                        let saved_amt = plan.final_saved.unwrap_or(0.0);
                        let pct = if plan.target_amount > 0.0 {
                            saved_amt / plan.target_amount * 100.0
                        } else {
                            100.0
                        };

                        // Muted static progress bar
                        let progress_frac = (pct / 100.0).clamp(0.0, 1.0) as f32;
                        let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 7.0), egui::Sense::hover());
                        ui.painter().rect_filled(bar_rect, Rounding::same(Theme::RADIUS_BAR), Theme::PANEL2);
                        let fill_w = (bar_rect.width() * progress_frac).clamp(0.0, bar_rect.width());
                        if fill_w > 0.0 {
                            let fill_rect = egui::Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, 7.0));
                            ui.painter().rect_filled(fill_rect, Rounding::same(Theme::RADIUS_BAR), Theme::FAINT);
                        }

                        ui.add_space(6.0);

                        // Row 2: Stat line — split into separate labels
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("Final: {}", Theme::format_pkr(saved_amt)))
                                    .font(Theme::font_sans(12.5))
                                    .color(Theme::MUTED),
                            );
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new(format!("/ {} target", Theme::format_pkr_whole(plan.target_amount)))
                                    .font(Theme::font_sans(12.5))
                                    .color(Theme::FAINT),
                            );
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new(format!("{:.0}%", pct))
                                    .font(Theme::font_sans(12.5))
                                    .color(Theme::MUTED),
                            );

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if let Some(closed_date) = plan.closed_at {
                                    ui.label(
                                        RichText::new(format!("Closed {}", Theme::format_date(&closed_date)))
                                            .font(Theme::font_sans(11.0))
                                            .color(Theme::FAINT),
                                    );
                                }
                            });
                        });

                        ui.add_space(6.0);

                        // Row 3: Action links
                        ui.horizontal(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.link(RichText::new("Delete").font(Theme::font_sans(12.0)).color(Theme::FAINT)).clicked() {
                                    state.plan_to_delete = Some(plan.id);
                                }
                            });
                        });
                    });

                    ui.add_space(Theme::CARD_GAP);
                }

                ui.add_space(Theme::PAGE_MARGIN);
            });
        });
    });
}

fn render_plan_modal(
    ui: &mut Ui,
    data: &mut AppData,
    state: &mut SavingsViewState,
    today: NaiveDate,
    data_changed: &mut bool,
) {
    let is_edit = state.editing_plan_id.is_some();
    let title = if is_edit { "Edit Savings Plan" } else { "Create New Savings Plan" };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ui.ctx(), |ui| {
            ui.set_min_width(400.0);
            ui.add_space(6.0);

            ui.label("Plan Name:");
            ui.text_edit_singleline(&mut state.name);

            ui.add_space(6.0);
            ui.label("Target Goal (PKR):");
            ui.text_edit_singleline(&mut state.target_amount);

            ui.add_space(6.0);
            ui.label("Deadline Date (M/D/YYYY):");
            ui.text_edit_singleline(&mut state.deadline_date);

            ui.add_space(6.0);
            ui.label("Monthly Income (PKR):");
            ui.text_edit_singleline(&mut state.monthly_income);

            ui.add_space(6.0);
            ui.label("Linked Cards to Track Spending:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut state.link_all_cards, true, "All Cards");
                ui.radio_value(&mut state.link_all_cards, false, "Specific Card");
            });

            if !state.link_all_cards {
                let card_name = data.cards
                    .iter()
                    .find(|c| Some(c.id) == state.selected_card_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Select Card");

                egui::ComboBox::from_id_salt("plan_card_select")
                    .selected_text(card_name)
                    .show_ui(ui, |ui| {
                        for c in &data.cards {
                            if ui.selectable_label(state.selected_card_id == Some(c.id), &c.name).clicked() {
                                state.selected_card_id = Some(c.id);
                            }
                        }
                    });
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            let target_parsed = Theme::parse_pkr_input(&state.target_amount).unwrap_or(0.0);
            let income_parsed = Theme::parse_pkr_input(&state.monthly_income).unwrap_or(0.0);
            let deadline_parsed = Theme::parse_date_input(&state.deadline_date);

            let linked_cards_filter = if state.link_all_cards {
                None
            } else {
                state.selected_card_id.map(|id| vec![id])
            };

            let prorated_sub = SubscriptionManager::total_monthly_cost(
                &data.subscriptions,
                linked_cards_filter.as_deref(),
            );

            if let Some(dl) = deadline_parsed {
                if target_parsed > 0.0 && income_parsed > 0.0 {
                    let override_val = if state.override_spending_limit {
                        Theme::parse_pkr_input(&state.spending_limit_override_val)
                    } else {
                        None
                    };

                    let (months_rem, req_savings, rec_limit, eff_limit, feasible, shortage) =
                        SavingsEngine::evaluate_plan_metrics(
                            today,
                            target_parsed,
                            dl,
                            income_parsed,
                            override_val,
                            prorated_sub,
                        );

                    egui::Frame::none()
                        .fill(Theme::BG_PANEL)
                        .stroke(Theme::panel_stroke())
                        .rounding(Rounding::same(Theme::RADIUS_SM))
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("Plan Calculation Preview:").strong().color(Theme::TEXT_PRIMARY));
                            ui.label(format!("• Months remaining: {:.1} months", months_rem));
                            ui.label(format!("• Required savings / mo: {}", Theme::format_pkr(req_savings)));
                            ui.label(format!("• Subscriptions cost: {} / mo", Theme::format_pkr(prorated_sub)));
                            let annual_reserve = SubscriptionManager::annual_reserve_monthly_needed(&data.subscriptions, linked_cards_filter.as_deref());
                            if annual_reserve > 0.0 {
                                ui.label(
                                    RichText::new(format!("  ↳ includes {}/mo reserved for yearly renewals (sinking fund)", Theme::format_pkr_whole(annual_reserve)))
                                        .font(Theme::font_sans(11.0))
                                        .color(Theme::CAT_SUBSCRIPTION),
                                );
                            }
                            ui.label(format!("• Recommended spending limit: {} / mo", Theme::format_pkr(rec_limit)));
                            if state.override_spending_limit {
                                ui.label(format!("• Active override limit: {} / mo", Theme::format_pkr(eff_limit)));
                            }

                            ui.add_space(4.0);
                            if !feasible {
                                ui.colored_label(
                                    Theme::EXPENSE,
                                    format!("⚠️ Unfeasible: Required savings exceed income by {}.", Theme::format_pkr(shortage)),
                                );
                            } else {
                                ui.colored_label(Theme::INCOME, "✔ Feasible within your monthly income.");
                            }
                        });

                    ui.add_space(6.0);
                    ui.checkbox(&mut state.override_spending_limit, "Override monthly spending limit");
                    if state.override_spending_limit {
                        ui.horizontal(|ui| {
                            ui.label("Custom Limit (PKR):");
                            ui.text_edit_singleline(&mut state.spending_limit_override_val);
                        });
                    }
                }
            }

            if let Some(err) = &state.modal_error {
                ui.colored_label(Theme::EXPENSE, err);
            }

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Save Plan").strong().color(Theme::ACCENT)).clicked() {
                    let name = state.name.trim().to_string();
                    if name.is_empty() {
                        state.modal_error = Some("Plan name cannot be empty.".to_string());
                        return;
                    }

                    if target_parsed <= 0.0 {
                        state.modal_error = Some("Target amount must be greater than 0.".to_string());
                        return;
                    }

                    let dl = match deadline_parsed {
                        Some(d) if d > today => d,
                        Some(_) => {
                            state.modal_error = Some("Deadline date must be in the future.".to_string());
                            return;
                        }
                        None => {
                            state.modal_error = Some("Invalid deadline date. Use M/D/YYYY (e.g. 9/4/2026).".to_string());
                            return;
                        }
                    };

                    if income_parsed <= 0.0 {
                        state.modal_error = Some("Monthly income must be a positive number.".to_string());
                        return;
                    }

                    let override_val = if state.override_spending_limit {
                        match Theme::parse_pkr_input(&state.spending_limit_override_val) {
                            Some(v) if v >= 0.0 => Some(v),
                            _ => {
                                state.modal_error = Some("Custom limit must be a valid number.".to_string());
                                return;
                            }
                        }
                    } else {
                        None
                    };

                    let linked_cards = if state.link_all_cards {
                        Vec::new()
                    } else if let Some(cid) = state.selected_card_id {
                        vec![cid]
                    } else {
                        Vec::new()
                    };

                    let now = chrono::Utc::now();
                    if let Some(edit_id) = state.editing_plan_id {
                        if let Some(p) = data.plans.iter_mut().find(|p| p.id == edit_id) {
                            p.name = name;
                            p.target_amount = target_parsed;
                            p.deadline = dl;
                            p.monthly_income = income_parsed;
                            p.spending_limit_override = override_val;
                            p.linked_card_ids = linked_cards;
                            p.deduct_overspending = state.deduct_overspending;
                            p.updated_at = now;
                        }
                    } else {
                        let new_plan = SavingsPlan {
                            id: Uuid::new_v4(),
                            name,
                            target_amount: target_parsed,
                            deadline: dl,
                            monthly_income: income_parsed,
                            spending_limit_override: override_val,
                            linked_card_ids: linked_cards,
                            created_at: today,
                            closed: false,
                            closed_at: None,
                            final_saved: None,
                            goal_met: None,
                            deduct_overspending: state.deduct_overspending,
                            updated_at: now,
                            deleted_at: None,
                        };
                        data.plans.push(new_plan);
                    }

                    *data_changed = true;
                    state.show_create_modal = false;
                }

                if ui.button("Cancel").clicked() {
                    state.show_create_modal = false;
                }
            });
        });
}
