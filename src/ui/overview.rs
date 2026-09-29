use crate::ledger::LedgerCalculator;
use crate::models::{AppData, BillingCycle};
use crate::savings::SavingsEngine;
use crate::subscriptions::SubscriptionManager;
use crate::ui::theme::{self, Theme};
use chrono::{Datelike, Duration, Local, NaiveDate};
use egui::{
    pos2, vec2, Align, Align2, Color32, Layout, Rect, RichText, Rounding, Stroke, Ui,
};

#[derive(Default)]
pub struct OverviewViewState;

pub fn render_overview_view(
    ui: &mut Ui,
    data: &AppData,
    _state: &mut OverviewViewState,
) {
    let today = Local::now().date_naive();
    let current_year = today.year();
    let current_month = today.month();

    // Subtle background grid texture
    let screen_rect = ui.max_rect();
    Theme::paint_grid_background(ui, screen_rect);

    let avail_width = (ui.available_width() - 2.0 * Theme::PAGE_MARGIN).max(0.0);
    let content_width = avail_width.min(Theme::CONTENT_MAX_WIDTH);
    let side_margin = Theme::PAGE_MARGIN + (avail_width - content_width) / 2.0;

    ui.add_space(Theme::CARD_GAP);

    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(side_margin);
            ui.vertical(|ui| {
                ui.set_width(content_width);
                ui.set_max_width(content_width);

                // ==========================================
                // 1. HERO ROW (~100px tall)
                // ==========================================
                // Calculate current net worth and 30-day history
                let non_deleted_cards: Vec<_> = data.cards.iter().filter(|c| c.deleted_at.is_none()).collect();
                let mut current_net_worth = 0.0;
                for card in &non_deleted_cards {
                    let rows = LedgerCalculator::compute_full_ledger(card, &data.transactions);
                    current_net_worth += rows.last().map(|r| r.running_balance).unwrap_or(card.opening_balance);
                }

                // Net worth 30 days ago and daily points
                let mut nw_30d_points = Vec::with_capacity(31);
                for day_offset in (0..=30).rev() {
                    let d = today - Duration::days(day_offset);
                    let mut nw_at_d = 0.0;
                    for card in &non_deleted_cards {
                        // Transactions up to date d
                        let txs_at_d: Vec<_> = data.transactions
                            .iter()
                            .filter(|t| t.deleted_at.is_none() && t.card_id == card.id && t.date <= d)
                            .cloned()
                            .collect();
                        let rows = LedgerCalculator::compute_full_ledger(card, &txs_at_d);
                        nw_at_d += rows.last().map(|r| r.running_balance).unwrap_or(card.opening_balance);
                    }
                    nw_30d_points.push(nw_at_d);
                }

                let nw_30d_ago = nw_30d_points.first().copied().unwrap_or(current_net_worth);
                let nw_delta = current_net_worth - nw_30d_ago;
                let nw_pct = if nw_30d_ago.abs() > 0.001 {
                    (nw_delta / nw_30d_ago.abs()) * 100.0
                } else {
                    0.0
                };

                egui::Frame::none()
                    .fill(Theme::PANEL)
                    .stroke(Theme::stroke_border())
                    .rounding(Rounding::same(Theme::RADIUS_CARD))
                    .inner_margin(egui::Margin::symmetric(24.0, 20.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_centered(|ui| {
                            // Net Worth hero number
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("Total Net Worth")
                                        .font(Theme::font_sans(12.0))
                                        .color(Theme::MUTED),
                                );
                                ui.add_space(2.0);
                                ui.label(
                                    RichText::new(Theme::format_pkr_whole(current_net_worth))
                                        .font(Theme::font_hero(40.0))
                                        .color(Theme::IN),
                                );
                            });

                            ui.add_space(16.0);

                            // Inline Sparkline: 120px wide x 40px tall
                            let spark_size = vec2(120.0, 40.0);
                            let (spark_rect, _) = ui.allocate_exact_size(spark_size, egui::Sense::hover());
                            paint_sparkline(ui, spark_rect, &nw_30d_points, if nw_delta >= 0.0 { Theme::IN } else { Theme::OUT });

                            ui.add_space(16.0);

                            // Delta Badge
                            let delta_color = if nw_delta >= 0.0 { Theme::IN } else { Theme::OUT };
                            let sign_str = if nw_delta >= 0.0 { "+" } else { "" };
                            let delta_text = format!(
                                "{}{} ({:+.1}%) vs 30d ago",
                                sign_str,
                                Theme::format_pkr_whole(nw_delta),
                                nw_pct
                            );

                            egui::Frame::none()
                                .fill(Theme::PANEL2)
                                .stroke(Stroke::new(1.0_f32, if nw_delta >= 0.0 { Theme::IN } else { Theme::OUT }))
                                .rounding(Rounding::same(Theme::RADIUS_BADGE))
                                .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(delta_text)
                                            .font(Theme::font_mono(12.0))
                                            .color(delta_color),
                                    );
                                });
                        });
                    });

                // ==========================================
                // 2. 16px GAP
                // ==========================================
                ui.add_space(16.0);

                // ==========================================
                // 3. STAT TILE ROW (4 equal-width tiles)
                // ==========================================
                // Calculate monthly stats
                let prev_month_date = today.checked_sub_months(chrono::Months::new(1)).unwrap_or(today);
                let prev_year = prev_month_date.year();
                let prev_month = prev_month_date.month();

                let mut income_curr = 0.0;
                let mut income_prev = 0.0;
                let mut spent_curr = 0.0;
                let mut spent_prev = 0.0;

                for tx in data.transactions.iter().filter(|t| t.deleted_at.is_none()) {
                    let y = tx.date.year();
                    let m = tx.date.month();
                    if y == current_year && m == current_month {
                        if tx.is_income {
                            income_curr += tx.amount;
                        } else {
                            spent_curr += tx.amount;
                        }
                    } else if y == prev_year && m == prev_month {
                        if tx.is_income {
                            income_prev += tx.amount;
                        } else {
                            spent_prev += tx.amount;
                        }
                    }
                }

                let sub_monthly_total = SubscriptionManager::total_monthly_cost(&data.subscriptions, None);

                let savings_rate_curr = if income_curr > 0.0 {
                    ((income_curr - spent_curr) / income_curr * 100.0).clamp(-100.0, 100.0)
                } else {
                    0.0
                };
                let savings_rate_prev = if income_prev > 0.0 {
                    ((income_prev - spent_prev) / income_prev * 100.0).clamp(-100.0, 100.0)
                } else {
                    0.0
                };

                let card_gap = Theme::CARD_GAP;
                let tile_width = (content_width - 3.0 * card_gap) / 4.0;

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(card_gap, 0.0);

                    // Tile 1: Income this month
                    let inc_arrow = if income_curr >= income_prev { "▲" } else { "▼" };
                    let inc_arrow_color = if income_curr >= income_prev { Theme::IN } else { Theme::OUT };
                    render_stat_tile(
                        ui,
                        tile_width,
                        "Income this month",
                        &Theme::format_pkr(income_curr),
                        Theme::IN,
                        Some((inc_arrow, inc_arrow_color)),
                        None,
                    );

                    // Tile 2: Spending this month (lower spending is better!)
                    let spent_arrow = if spent_curr <= spent_prev { "▼" } else { "▲" };
                    let spent_arrow_color = if spent_curr <= spent_prev { Theme::IN } else { Theme::OUT };
                    render_stat_tile(
                        ui,
                        tile_width,
                        "Spending this month",
                        &Theme::format_pkr(spent_curr),
                        Theme::OUT,
                        Some((spent_arrow, spent_arrow_color)),
                        None,
                    );

                    // Tile 3: Subscriptions (monthly prorated + annual reserve info)
                    let annual_reserve = SubscriptionManager::annual_reserve_monthly_needed(&data.subscriptions, None);
                    let sub_reserve_subtext = if annual_reserve > 0.0 {
                        Some((format!("incl. {}/mo reserve", Theme::format_pkr_whole(annual_reserve)), Theme::CAT_SUBSCRIPTION))
                    } else {
                        None
                    };
                    render_stat_tile(
                        ui,
                        tile_width,
                        "Active subscriptions",
                        &Theme::format_pkr(sub_monthly_total),
                        Theme::TEXT,
                        None,
                        sub_reserve_subtext.as_ref().map(|(s, c)| (s.as_str(), *c)),
                    );

                    // Tile 4: Savings rate this month
                    let sr_arrow = if savings_rate_curr >= savings_rate_prev { "▲" } else { "▼" };
                    let sr_arrow_color = if savings_rate_curr >= savings_rate_prev { Theme::IN } else { Theme::OUT };
                    let sr_text = format!("{:.1}%", savings_rate_curr);
                    let sr_color = if savings_rate_curr >= 0.0 { Theme::IN } else { Theme::OUT };
                    render_stat_tile(
                        ui,
                        tile_width,
                        "Savings rate this month",
                        &sr_text,
                        sr_color,
                        Some((sr_arrow, sr_arrow_color)),
                        None,
                    );
                });

                // ==========================================
                // 4. 16px GAP
                // ==========================================
                ui.add_space(16.0);

                // ==========================================
                // 5. CATEGORY BREAKDOWN ROW (Donut + Legend)
                // ==========================================
                // Spending breakdown by category for current month
                let mut cat_breakdown: std::collections::BTreeMap<String, f64> = std::collections::BTreeMap::new();
                for tx in data.transactions.iter().filter(|t| t.deleted_at.is_none() && !t.is_income) {
                    if tx.date.year() == current_year && tx.date.month() == current_month {
                        *cat_breakdown.entry(tx.category.clone()).or_insert(0.0) += tx.amount;
                    }
                }

                let mut cat_list: Vec<(String, f64, Color32)> = cat_breakdown
                    .into_iter()
                    .map(|(cat, amt)| {
                        let c = theme::category_color(&cat);
                        (cat, amt, c)
                    })
                    .collect();
                cat_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                let total_cat_spending: f64 = cat_list.iter().map(|(_, a, _)| *a).sum();

                egui::Frame::none()
                    .fill(Theme::PANEL)
                    .stroke(Theme::stroke_border())
                    .rounding(Rounding::same(Theme::RADIUS_CARD))
                    .inner_margin(egui::Margin::symmetric(22.0, 18.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        ui.label(
                            RichText::new("Spending by Category (This Month)")
                                .font(Theme::font_head(14.0))
                                .strong()
                                .color(Theme::TEXT),
                        );
                        ui.add_space(14.0);

                        let avail_chart_w = ui.available_width();
                        if avail_chart_w >= 600.0 {
                            ui.horizontal_top(|ui| {
                                // ~40% donut chart on left
                                let donut_w = (avail_chart_w * 0.38).clamp(160.0, 240.0);
                                let (donut_rect, _) = ui.allocate_exact_size(vec2(donut_w, 180.0), egui::Sense::hover());
                                paint_donut_chart(ui, donut_rect, &cat_list, total_cat_spending);

                                ui.add_space(20.0);

                                // ~60% legend list on right
                                ui.vertical(|ui| {
                                    render_category_legend(ui, &cat_list, total_cat_spending);
                                });
                            });
                        } else {
                            // Stacked for narrow sizes
                            let (donut_rect, _) = ui.allocate_exact_size(vec2(avail_chart_w, 170.0), egui::Sense::hover());
                            paint_donut_chart(ui, donut_rect, &cat_list, total_cat_spending);
                            ui.add_space(12.0);
                            render_category_legend(ui, &cat_list, total_cat_spending);
                        }
                    });

                // ==========================================
                // 6. 16px GAP
                // ==========================================
                ui.add_space(16.0);

                // ==========================================
                // 7. UPCOMING ROW (List up to 5 items)
                // ==========================================
                struct UpcomingItem {
                    name: String,
                    date: NaiveDate,
                    amount: f64,
                    color: Color32,
                    is_plan: bool,
                }

                let mut upcoming: Vec<UpcomingItem> = Vec::new();

                // Subscriptions due in next 30 days
                for sub in data.subscriptions.iter().filter(|s| s.deleted_at.is_none() && !s.paused) {
                    let days_left = (sub.next_due_date - today).num_days();
                    if days_left >= 0 && days_left <= 30 {
                        upcoming.push(UpcomingItem {
                            name: sub.name.clone(),
                            date: sub.next_due_date,
                            amount: sub.amount,
                            color: Theme::CAT_SUBSCRIPTION,
                            is_plan: false,
                        });
                    }
                }

                // Savings plans deadline in next 30 days
                for plan in data.plans.iter().filter(|p| p.deleted_at.is_none() && !p.closed) {
                    let days_left = (plan.deadline - today).num_days();
                    if days_left >= 0 && days_left <= 30 {
                        let prorated_sub = SubscriptionManager::total_monthly_cost(
                            &data.subscriptions,
                            if plan.linked_card_ids.is_empty() { None } else { Some(&plan.linked_card_ids) },
                        );
                        let calc = SavingsEngine::compute_progress(plan, &data.transactions, prorated_sub, today, false);
                        let status_col = if !calc.is_feasible || calc.percent_reached < 50.0 {
                            Theme::STATUS_BEHIND
                        } else if calc.percent_reached >= 90.0 {
                            Theme::STATUS_AHEAD
                        } else {
                            Theme::STATUS_ON_TRACK
                        };

                        upcoming.push(UpcomingItem {
                            name: format!("{} (Target)", plan.name),
                            date: plan.deadline,
                            amount: plan.target_amount,
                            color: status_col,
                            is_plan: true,
                        });
                    }
                }

                upcoming.sort_by_key(|item| item.date);
                upcoming.truncate(5);

                egui::Frame::none()
                    .fill(Theme::PANEL)
                    .stroke(Theme::stroke_border())
                    .rounding(Rounding::same(Theme::RADIUS_CARD))
                    .inner_margin(egui::Margin::symmetric(22.0, 18.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        ui.label(
                            RichText::new("Upcoming (Next 30 Days)")
                                .font(Theme::font_head(13.0))
                                .color(Theme::MUTED),
                        );
                        ui.add_space(10.0);

                        if upcoming.is_empty() {
                            ui.label(
                                RichText::new("No upcoming subscription bills or savings plan deadlines in the next 30 days.")
                                    .font(Theme::font_sans(12.5))
                                    .color(Theme::FAINT),
                            );
                        } else {
                            for (idx, item) in upcoming.iter().enumerate() {
                                if idx > 0 {
                                    ui.add_space(4.0);
                                    let (line_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), egui::Sense::hover());
                                    ui.painter().line_segment([line_rect.left_top(), line_rect.right_top()], Theme::stroke_border());
                                    ui.add_space(4.0);
                                }

                                ui.horizontal(|ui| {
                                    // Colored dot (subscription vs plan deadline)
                                    let (dot_rect, _) = ui.allocate_exact_size(vec2(8.0, 8.0), egui::Sense::hover());
                                    ui.painter().circle_filled(dot_rect.center(), 3.5, item.color);
                                    ui.add_space(6.0);

                                    // Item name
                                    let max_name_len = 32;
                                    let display_name = if item.name.len() > max_name_len {
                                        format!("{}…", &item.name[..max_name_len])
                                    } else {
                                        item.name.clone()
                                    };
                                    ui.label(
                                        RichText::new(display_name)
                                            .font(Theme::font_sans(13.0))
                                            .color(Theme::TEXT),
                                    );

                                    // Right items: Date and Amount
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        let amount_color = if item.is_plan { Theme::IN } else { Theme::OUT };
                                        ui.label(
                                            RichText::new(Theme::format_pkr(item.amount))
                                                .font(Theme::font_mono(13.0))
                                                .color(amount_color),
                                        );

                                        ui.add_space(20.0);

                                        let days_left = (item.date - today).num_days();
                                        let due_str = if days_left == 0 {
                                            "Today".to_string()
                                        } else if days_left == 1 {
                                            "Tomorrow".to_string()
                                        } else {
                                            format!("in {}d ({})", days_left, item.date.format("%b %d"))
                                        };
                                        ui.label(
                                            RichText::new(due_str)
                                                .font(Theme::font_sans(12.0))
                                                .color(Theme::MUTED),
                                        );
                                    });
                                });
                            }
                        }
                    });

                // ==========================================
                // 8. ANNUAL RENEWALS · SINKING FUND RESERVE
                // ==========================================
                let yearly_subs: Vec<_> = data
                    .subscriptions
                    .iter()
                    .filter(|s| s.deleted_at.is_none() && !s.paused && s.cycle == BillingCycle::Yearly)
                    .collect();

                if !yearly_subs.is_empty() {
                    ui.add_space(16.0);
                    egui::Frame::none()
                        .fill(Theme::PANEL)
                        .stroke(Theme::stroke_border())
                        .rounding(Rounding::same(Theme::RADIUS_CARD))
                        .inner_margin(egui::Margin::symmetric(22.0, 18.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Annual Renewals · Sinking Fund Reserve")
                                        .font(Theme::font_head(13.0))
                                        .color(Theme::MUTED),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let total_monthly = SubscriptionManager::annual_reserve_monthly_needed(&data.subscriptions, None);
                                    ui.label(
                                        RichText::new(format!("Need {} / mo", Theme::format_pkr_whole(total_monthly)))
                                            .font(Theme::font_mono(12.0))
                                            .color(Theme::CAT_SUBSCRIPTION),
                                    );
                                });
                            });
                            ui.add_space(12.0);

                            for (idx, sub) in yearly_subs.iter().enumerate() {
                                if idx > 0 {
                                    ui.add_space(6.0);
                                    let (line_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), egui::Sense::hover());
                                    ui.painter().line_segment([line_rect.left_top(), line_rect.right_top()], Theme::stroke_border());
                                    ui.add_space(6.0);
                                }

                                if let Some(status) = SubscriptionManager::yearly_reserve_status(sub, today) {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(&sub.name)
                                                .font(Theme::font_sans(13.0))
                                                .color(Theme::TEXT),
                                        );

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            let text_color = if status.months_remaining <= 1 {
                                                Theme::STATUS_BEHIND
                                            } else {
                                                Theme::TEXT
                                            };
                                            ui.label(
                                                RichText::new(format!(
                                                    "Saved {} / {} · {}",
                                                    Theme::format_pkr_whole(status.saved_amount),
                                                    Theme::format_pkr_whole(status.total_cost),
                                                    status.due_text
                                                ))
                                                .font(Theme::font_mono(12.0))
                                                .color(text_color),
                                            );
                                        });
                                    });

                                    ui.add_space(5.0);
                                    let (pb_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 4.0), egui::Sense::hover());
                                    ui.painter().rect_filled(pb_rect, Rounding::same(2.0), Theme::BORDER);
                                    let mut filled_rect = pb_rect;
                                    filled_rect.set_width(pb_rect.width() * (status.progress_ratio as f32).clamp(0.0, 1.0));
                                    let bar_color = if status.months_remaining <= 1 {
                                        Theme::STATUS_BEHIND
                                    } else if status.progress_ratio >= 0.75 {
                                        Theme::STATUS_AHEAD
                                    } else {
                                        Theme::CAT_SUBSCRIPTION
                                    };
                                    ui.painter().rect_filled(filled_rect, Rounding::same(2.0), bar_color);
                                }
                            }
                        });
                }

                ui.add_space(Theme::CARD_GAP);
            });
        });
    });
}

fn render_stat_tile(
    ui: &mut Ui,
    width: f32,
    label: &str,
    val_text: &str,
    val_color: Color32,
    arrow: Option<(&str, Color32)>,
    sub_text: Option<(&str, Color32)>,
) {
    egui::Frame::none()
        .fill(Theme::PANEL)
        .stroke(Theme::stroke_border())
        .rounding(Rounding::same(Theme::RADIUS_CARD))
        .inner_margin(egui::Margin::symmetric(16.0, 14.0))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.set_max_width(width);

            ui.label(
                RichText::new(label)
                    .font(Theme::font_sans(11.5))
                    .color(Theme::MUTED),
            );
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(val_text)
                        .font(Theme::font_mono(16.5))
                        .color(val_color),
                );

                if let Some((arr, arr_color)) = arrow {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(arr)
                            .font(Theme::font_sans(10.0))
                            .color(arr_color),
                    );
                }
            });

            if let Some((sub, sub_color)) = sub_text {
                ui.add_space(2.0);
                ui.label(RichText::new(sub).font(Theme::font_sans(10.5)).color(sub_color));
            }
        });
}

fn paint_sparkline(ui: &mut Ui, rect: Rect, data: &[f64], line_color: Color32) {
    if data.is_empty() {
        return;
    }

    let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (max_val - min_val).max(1.0);

    let n = data.len();
    let dx = if n > 1 { rect.width() / (n - 1) as f32 } else { rect.width() };

    let mut points = Vec::with_capacity(n);
    for (i, &val) in data.iter().enumerate() {
        let norm_y = ((val - min_val) / range) as f32;
        let x = rect.left() + i as f32 * dx;
        // Invert Y so max is at top of rect
        let y = rect.bottom() - 4.0 - norm_y * (rect.height() - 8.0);
        points.push(pos2(x, y));
    }

    // Faint subtle background area under sparkline
    if points.len() >= 2 {
        let mut fill_points = points.clone();
        fill_points.push(pos2(rect.right(), rect.bottom()));
        fill_points.push(pos2(rect.left(), rect.bottom()));
        let fill_color = Color32::from_rgba_premultiplied(
            (line_color.r() as f32 * 0.12) as u8,
            (line_color.g() as f32 * 0.12) as u8,
            (line_color.b() as f32 * 0.12) as u8,
            30,
        );
        ui.painter().add(egui::Shape::convex_polygon(fill_points, fill_color, Stroke::NONE));
    }

    // Draw the sparkline itself
    for window in points.windows(2) {
        ui.painter().line_segment([window[0], window[1]], Stroke::new(1.8_f32, line_color));
    }

    // End point highlight
    if let Some(&last_pt) = points.last() {
        ui.painter().circle_filled(last_pt, 2.5, line_color);
    }
}

fn paint_donut_chart(
    ui: &mut Ui,
    rect: Rect,
    categories: &[(String, f64, Color32)],
    total: f64,
) {
    let center = rect.center();
    let outer_r = (rect.height() / 2.0 - 10.0).min(rect.width() / 2.0 - 10.0).clamp(40.0, 75.0);
    let inner_r = outer_r * 0.58;

    if total <= 0.0 || categories.is_empty() {
        // Empty placeholder ring
        let painter = ui.painter();
        let stroke = Stroke::new(outer_r - inner_r, Theme::PANEL2);
        let mid_r = (outer_r + inner_r) / 2.0;
        painter.circle_stroke(center, mid_r, stroke);
        painter.text(
            center,
            Align2::CENTER_CENTER,
            "No spending",
            Theme::font_sans(11.0),
            Theme::FAINT,
        );
        return;
    }

    let mut start_angle = -std::f32::consts::FRAC_PI_2; // Start from 12 o'clock

    for (_, amt, color) in categories {
        let frac = (*amt / total) as f32;
        let sweep = frac * std::f32::consts::TAU;
        let end_angle = start_angle + sweep;

        // Tessellate arc slice
        let steps = (sweep.abs() * 20.0).ceil().max(2.0) as usize;
        let d_theta = sweep / steps as f32;

        let mut slice_mesh = egui::epaint::Mesh::default();

        for i in 0..=steps {
            let theta = start_angle + i as f32 * d_theta;
            let (sin_t, cos_t) = theta.sin_cos();
            let p_out = center + vec2(cos_t * outer_r, sin_t * outer_r);
            let p_in = center + vec2(cos_t * inner_r, sin_t * inner_r);

            let v_out = slice_mesh.vertices.len() as u32;
            slice_mesh.colored_vertex(p_out, *color);
            let v_in = slice_mesh.vertices.len() as u32;
            slice_mesh.colored_vertex(p_in, *color);

            if i > 0 {
                // Two triangles forming quad between step i-1 and step i
                let prev_out = v_out - 2;
                let prev_in = v_in - 2;
                slice_mesh.add_triangle(prev_out, v_out, prev_in);
                slice_mesh.add_triangle(prev_in, v_out, v_in);
            }
        }

        ui.painter().add(egui::Shape::mesh(slice_mesh));
        start_angle = end_angle;
    }

    // Inner cutout text
    ui.painter().text(
        center - vec2(0.0, 7.0),
        Align2::CENTER_CENTER,
        "Total Spent",
        Theme::font_sans(10.0),
        Theme::MUTED,
    );
    ui.painter().text(
        center + vec2(0.0, 7.0),
        Align2::CENTER_CENTER,
        Theme::format_pkr_whole(total),
        Theme::font_mono(11.5),
        Theme::TEXT,
    );
}

fn render_category_legend(ui: &mut Ui, categories: &[(String, f64, Color32)], total: f64) {
    if categories.is_empty() {
        ui.label(
            RichText::new("No expenses recorded for this month yet.")
                .font(Theme::font_sans(12.0))
                .color(Theme::FAINT),
        );
        return;
    }

    for (cat_name, amount, color) in categories {
        let pct = if total > 0.0 { (*amount / total) * 100.0 } else { 0.0 };

        ui.horizontal(|ui| {
            // Color swatch (10px square)
            let (swatch_rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), egui::Sense::hover());
            ui.painter().rect_filled(swatch_rect, Rounding::same(2.0), *color);

            ui.add_space(8.0);

            // Category name
            ui.label(
                RichText::new(cat_name)
                    .font(Theme::font_sans(12.5))
                    .color(Theme::TEXT),
            );

            // Right-aligned Amount and Percentage
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Sub-column: Percentage (muted)
                ui.add_sized(
                    vec2(44.0, 18.0),
                    egui::Label::new(
                        RichText::new(format!("{:.1}%", pct))
                            .font(Theme::font_sans(11.5))
                            .color(Theme::MUTED),
                    ),
                );

                ui.add_space(8.0);

                // Sub-column: Amount (mono, right-aligned)
                Theme::money_label(ui, &Theme::format_pkr(*amount), Theme::TEXT, 12.5);
            });
        });
        ui.add_space(4.0);
    }
}
