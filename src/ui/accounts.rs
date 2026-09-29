use crate::ledger::LedgerCalculator;
use crate::models::{AppData, Card, Transaction};
use crate::ui::theme::Theme;
use chrono::{Datelike, Local, NaiveDate};
use egui::{pos2, vec2, Align, Color32, Layout, RichText, Rounding, Stroke, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpreadsheetCol {
    Description = 0,
    Category = 1,
    MoneyIn = 2,
    MoneyOut = 3,
}

#[derive(Default)]
pub struct AccountsViewState {
    pub selected_card_id: Option<Uuid>,
    pub filter_preset: DateFilterPresetWrapper,
    pub custom_date_from: String,
    pub custom_date_to: String,
    pub selected_category: String,
    pub search_query: String,
    pub sort_descending: bool,

    // Spreadsheet inline editing & navigation state
    pub active_edit: Option<(Uuid, SpreadsheetCol)>,
    pub edit_str: String,
    pub edit_just_focused: bool,

    // Modals
    pub show_add_card: bool,
    pub new_card_name: String,
    pub new_card_opening_balance: String,

    pub show_rename_card: bool,
    pub rename_card_name: String,

    pub card_to_delete: Option<Uuid>,

    pub show_tx_modal: bool,
    pub editing_tx_id: Option<Uuid>,
    pub tx_is_income: bool,
    pub tx_date: String,
    pub tx_description: String,
    pub tx_category: String,
    pub tx_amount: String,
    pub tx_error: Option<String>,

    pub show_opening_balance_modal: bool,
    pub op_bal_amount: String,
    pub op_bal_description: String,
    pub op_bal_date: String,
    pub op_bal_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFilterPresetWrapper {
    All,
    ThisMonth,
    Last30Days,
    Custom,
}

impl Default for DateFilterPresetWrapper {
    fn default() -> Self {
        Self::All
    }
}

impl AccountsViewState {
    pub fn open_add_tx(&mut self, is_income: bool) {
        self.show_tx_modal = true;
        self.editing_tx_id = None;
        self.tx_is_income = is_income;
        self.tx_date = Theme::format_input_date(&Local::now().date_naive());
        self.tx_description.clear();
        self.tx_category = if is_income {
            "Income".to_string()
        } else {
            "Food".to_string()
        };
        self.tx_amount.clear();
        self.tx_error = None;
    }

    pub fn open_edit_tx(&mut self, tx: &Transaction) {
        self.show_tx_modal = true;
        self.editing_tx_id = Some(tx.id);
        self.tx_is_income = tx.is_income;
        self.tx_date = Theme::format_input_date(&tx.date);
        self.tx_description = tx.description.clone();
        self.tx_category = tx.category.clone();
        self.tx_amount = format!("{:.2}", tx.amount);
        self.tx_error = None;
    }

    pub fn open_edit_opening_balance(&mut self, card: &Card, default_date: NaiveDate) {
        self.show_opening_balance_modal = true;
        self.op_bal_amount = format!("{:.2}", card.opening_balance);
        self.op_bal_description = card.opening_balance_description.clone();
        let d = card.opening_balance_date.unwrap_or(default_date);
        self.op_bal_date = Theme::format_input_date(&d);
        self.op_bal_error = None;
    }
}

pub fn render_accounts_view(
    ui: &mut Ui,
    data: &mut AppData,
    state: &mut AccountsViewState,
    data_changed: &mut bool,
) {
    if data.cards.is_empty() {
        ui.label("No cards available. Please add a card.");
        return;
    }

    // Ensure valid selected card
    if state.selected_card_id.is_none() || !data.cards.iter().any(|c| Some(c.id) == state.selected_card_id) {
        state.selected_card_id = Some(data.cards[0].id);
    }

    let today = Local::now().date_naive();
    let mut active_card_id = state.selected_card_id.unwrap();

    egui::Frame::none()
        .inner_margin(egui::Margin {
            left: Theme::PAGE_MARGIN,
            right: Theme::PAGE_MARGIN,
            top: Theme::CARD_GAP,
            bottom: Theme::PAGE_MARGIN,
        })
        .show(ui, |ui| {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(Theme::CARD_GAP, 8.0);

        let mut card_to_select = None;

        for card in &data.cards {
            let is_selected = state.selected_card_id == Some(card.id);

            let bg_color = if is_selected {
                Theme::PANEL2
            } else {
                Theme::PANEL
            };

            let stroke = if is_selected {
                Theme::stroke_accent_dim()
            } else {
                Theme::stroke_border()
            };

            let text_color = if is_selected {
                Theme::TEXT
            } else {
                Theme::MUTED
            };

            let mut job = egui::text::LayoutJob::default();
            if card.is_primary {
                job.append(
                    "★ ",
                    0.0,
                    egui::TextFormat {
                        font_id: Theme::font_sans(11.0),
                        color: Theme::STAR,
                        ..Default::default()
                    },
                );
            }
            job.append(
                &card.name,
                0.0,
                egui::TextFormat {
                    font_id: Theme::font_sans(13.0),
                    color: text_color,
                    ..Default::default()
                },
            );

            let text_galley = ui.painter().layout_job(job);
            let desired_size = text_galley.size() + Vec2::new(28.0, 14.0);
            let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

            let final_bg = if is_selected {
                Theme::PANEL2
            } else if response.hovered() {
                Theme::PANEL2
            } else {
                bg_color
            };

            ui.painter().rect(rect, Rounding::same(Theme::RADIUS_TAB), final_bg, stroke);

            let text_pos = rect.center() - text_galley.size() / 2.0;
            ui.painter().galley(text_pos, text_galley, text_color);

            if response.clicked() {
                card_to_select = Some(card.id);
            }
        }

        if let Some(id) = card_to_select {
            state.selected_card_id = Some(id);
        }

        // Dashed "+ Add card" pill button at the end
        let add_galley = ui.painter().layout_no_wrap(
            "+ Add card".to_string(),
            Theme::font_sans(13.0),
            Theme::ACCENT,
        );
        let add_size = add_galley.size() + Vec2::new(28.0, 14.0);
        let (add_rect, add_resp) = ui.allocate_exact_size(add_size, egui::Sense::click());

        let add_bg = if add_resp.hovered() { Theme::PANEL2 } else { Color32::TRANSPARENT };
        ui.painter().rect(
            add_rect,
            Rounding::same(Theme::RADIUS_TAB),
            add_bg,
            Theme::dashed_stroke(),
        );
        let add_text_pos = add_rect.center() - add_galley.size() / 2.0;
        ui.painter().galley(add_text_pos, add_galley, Theme::ACCENT);

        if add_resp.clicked() {
            state.show_add_card = true;
            state.new_card_name.clear();
            state.new_card_opening_balance = "0".to_string();
        }
    });

    ui.add_space(Theme::CARD_GAP);

    // Active Card Data
    active_card_id = state.selected_card_id.unwrap();
    let current_card_idx = data.cards.iter().position(|c| c.id == active_card_id).unwrap_or(0);
    let current_card = data.cards[current_card_idx].clone();

    // Compute ledger figures
    let full_rows = LedgerCalculator::compute_full_ledger(&current_card, &data.transactions);
    let current_balance = full_rows.last().map(|r| r.running_balance).unwrap_or(current_card.opening_balance);

    let this_month_in: f64 = data.transactions
        .iter()
        .filter(|t| t.card_id == current_card.id && t.date.year() == today.year() && t.date.month() == today.month() && t.is_income)
        .map(|t| t.amount)
        .sum();

    let this_month_out: f64 = data.transactions
        .iter()
        .filter(|t| t.card_id == current_card.id && t.date.year() == today.year() && t.date.month() == today.month() && !t.is_income)
        .map(|t| t.amount)
        .sum();

    // Filter calculations
    let (filter_from, filter_to) = match state.filter_preset {
        DateFilterPresetWrapper::All => (None, None),
        DateFilterPresetWrapper::ThisMonth => (
            NaiveDate::from_ymd_opt(today.year(), today.month(), 1),
            Some(today),
        ),
        DateFilterPresetWrapper::Last30Days => (
            Some(today - chrono::Duration::days(30)),
            Some(today),
        ),
        DateFilterPresetWrapper::Custom => {
            let f = Theme::parse_date_input(&state.custom_date_from);
            let t = Theme::parse_date_input(&state.custom_date_to);
            (f, t)
        }
    };

    let cat_filter = if state.selected_category.is_empty() { None } else { Some(state.selected_category.as_str()) };
    let displayed_rows = LedgerCalculator::filter_and_sort_ledger(
        &full_rows,
        cat_filter,
        filter_from,
        filter_to,
        &state.search_query,
        state.sort_descending,
    );

    // --- 2. SINGLE UNIFIED PANEL (.panel) FOR SUMMARY + TOOLBAR + TABLE ---
    egui::Frame::none()
        .fill(Theme::PANEL)
        .stroke(Theme::stroke_border())
        .rounding(Rounding::same(Theme::RADIUS_PANEL))
        .show(ui, |ui| {
            // A. SUMMARY SECTION (padding: 20px 22px; gap: 44px)
            egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(22.0, Theme::CARD_PADDING))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Card Name & Primary badge
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&current_card.name)
                                    .font(Theme::font_head(19.0))
                                    .strong()
                                    .color(Theme::TEXT),
                            );

                            ui.add_space(8.0);

                            if current_card.is_primary {
                                let badge_text = "★ Primary · Earning account";
                                let badge_galley = ui.painter().layout_no_wrap(
                                    badge_text.to_string(),
                                    Theme::font_sans(11.0),
                                    Theme::STAR,
                                );
                                let badge_size = badge_galley.size() + Vec2::new(16.0, 6.0);
                                let (b_rect, _) = ui.allocate_exact_size(badge_size, egui::Sense::hover());
                                ui.painter().rect(
                                    b_rect,
                                    Rounding::same(Theme::RADIUS_BADGE),
                                    Theme::BADGE_BG,
                                    Stroke::NONE,
                                );
                                let b_pos = b_rect.center() - badge_galley.size() / 2.0;
                                ui.painter().galley(b_pos, badge_galley, Theme::STAR);
                            } else {
                                let set_p_galley = ui.painter().layout_no_wrap(
                                    "Set as Primary".to_string(),
                                    Theme::font_sans(11.0),
                                    Theme::MUTED,
                                );
                                let set_p_size = set_p_galley.size() + Vec2::new(14.0, 6.0);
                                let (p_rect, p_resp) = ui.allocate_exact_size(set_p_size, egui::Sense::click());
                                let p_bg = if p_resp.hovered() { Theme::PANEL2 } else { Color32::TRANSPARENT };
                                ui.painter().rect(p_rect, Rounding::same(Theme::RADIUS_BADGE), p_bg, Theme::stroke_border());
                                let p_pos = p_rect.center() - set_p_galley.size() / 2.0;
                                ui.painter().galley(p_pos, set_p_galley, Theme::MUTED);
                                if p_resp.clicked() {
                                    for c in &mut data.cards {
                                        c.is_primary = c.id == current_card.id;
                                    }
                                    *data_changed = true;
                                }
                            }
                        });

                        ui.add_space(32.0);

                        // Running Balance
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Running balance").font(Theme::font_sans(11.5)).color(Theme::MUTED));
                            let bal_color = if current_balance >= 0.0 { Theme::TEXT } else { Theme::OUT };
                            Theme::money_label(ui, &Theme::format_pkr(current_balance), bal_color, 16.0);
                        });

                        ui.add_space(28.0);

                        // Income This Month
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Income this month").font(Theme::font_sans(11.5)).color(Theme::MUTED));
                            Theme::money_label(ui, &Theme::format_pkr(this_month_in), Theme::IN, 16.0);
                        });

                        ui.add_space(28.0);

                        // Spending This Month
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Spending this month").font(Theme::font_sans(11.5)).color(Theme::MUTED));
                            Theme::money_label(ui, &Theme::format_pkr(this_month_out), Theme::OUT, 16.0);
                        });

                        ui.add_space(28.0);

                        // Opening Balance
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Opening balance").font(Theme::font_sans(11.5)).color(Theme::MUTED));
                            ui.horizontal(|ui| {
                                Theme::money_label(ui, &Theme::format_pkr(current_card.opening_balance), Theme::TEXT, 16.0);
                                let edit_btn = egui::Frame::none()
                                    .stroke(Theme::stroke_border())
                                    .rounding(Rounding::same(Theme::RADIUS_BTN))
                                    .inner_margin(egui::Margin::symmetric(10.0, 4.0));
                                if edit_btn.show(ui, |ui| {
                                    ui.label(RichText::new("Edit").font(Theme::font_sans(12.0)).color(Theme::MUTED));
                                }).response.interact(egui::Sense::click()).clicked() {
                                    state.open_edit_opening_balance(&current_card, today);
                                }
                            });
                        });

                        // Right aligned Rename & Delete
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if data.cards.len() > 1 {
                                let del_btn = egui::Frame::none()
                                    .stroke(Theme::stroke_border())
                                    .rounding(Rounding::same(Theme::RADIUS_BTN))
                                    .inner_margin(egui::Margin::symmetric(10.0, 5.0));
                                if del_btn.show(ui, |ui| {
                                    ui.label(RichText::new("Delete").font(Theme::font_sans(12.5)).color(Theme::OUT));
                                }).response.interact(egui::Sense::click()).clicked() {
                                    state.card_to_delete = Some(current_card.id);
                                }
                                ui.add_space(6.0);
                            }

                            let rename_btn = egui::Frame::none()
                                .stroke(Theme::stroke_border())
                                .rounding(Rounding::same(Theme::RADIUS_BTN))
                                .inner_margin(egui::Margin::symmetric(12.0, 6.0));
                            if rename_btn.show(ui, |ui| {
                                ui.label(RichText::new("Rename").font(Theme::font_sans(12.5)).color(Theme::MUTED));
                            }).response.interact(egui::Sense::click()).clicked() {
                                state.show_rename_card = true;
                                state.rename_card_name = current_card.name.clone();
                            }
                        });
                    });
                });

            // 1px Border separator
            let (sep_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
            ui.painter().line_segment([sep_rect.left_top(), sep_rect.right_top()], Theme::stroke_border());

            // B. TOOLBAR SECTION (padding: 14px 22px; gap: 10px)
            egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(22.0, 12.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // "+ Add income" button (.btn.income: background rgba(63,205,168,.12); color var(--in))
                        let in_galley = ui.painter().layout_no_wrap(
                            "+ Add income".to_string(),
                            Theme::font_sans(13.0),
                            Theme::IN,
                        );
                        let in_size = in_galley.size() + Vec2::new(24.0, 12.0);
                        let (in_rect, in_resp) = ui.allocate_exact_size(in_size, egui::Sense::click());
                        let in_bg = if in_resp.hovered() {
                            Color32::from_rgba_premultiplied(14, 45, 36, 50)
                        } else {
                            Theme::IN_BTN_BG
                        };
                        ui.painter().rect(
                            in_rect,
                            Rounding::same(Theme::RADIUS_BTN),
                            in_bg,
                            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(15, 50, 40, 60)),
                        );
                        let in_pos = in_rect.center() - in_galley.size() / 2.0;
                        ui.painter().galley(in_pos, in_galley, Theme::IN);
                        if in_resp.clicked() {
                            state.open_add_tx(true);
                        }

                        ui.add_space(8.0);

                        // "+ Add expense" button (.btn.expense: background rgba(242,114,107,.12); color var(--out))
                        let out_galley = ui.painter().layout_no_wrap(
                            "+ Add expense".to_string(),
                            Theme::font_sans(13.0),
                            Theme::OUT,
                        );
                        let out_size = out_galley.size() + Vec2::new(24.0, 12.0);
                        let (out_rect, out_resp) = ui.allocate_exact_size(out_size, egui::Sense::click());
                        let out_bg = if out_resp.hovered() {
                            Color32::from_rgba_premultiplied(46, 22, 20, 50)
                        } else {
                            Theme::OUT_BTN_BG
                        };
                        ui.painter().rect(
                            out_rect,
                            Rounding::same(Theme::RADIUS_BTN),
                            out_bg,
                            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(55, 26, 24, 60)),
                        );
                        let out_pos = out_rect.center() - out_galley.size() / 2.0;
                        ui.painter().galley(out_pos, out_galley, Theme::OUT);
                        if out_resp.clicked() {
                            state.open_add_tx(false);
                        }

                        ui.add_space(14.0);

                        // Segmented control (.seg: background var(--panel2); border-radius 7px; padding 2px)
                        let presets = [
                            (DateFilterPresetWrapper::All, "All"),
                            (DateFilterPresetWrapper::ThisMonth, "This month"),
                            (DateFilterPresetWrapper::Last30Days, "Last 30d"),
                            (DateFilterPresetWrapper::Custom, "Custom"),
                        ];

                        egui::Frame::none()
                            .fill(Theme::PANEL2)
                            .rounding(Rounding::same(Theme::RADIUS_BTN))
                            .inner_margin(egui::Margin::same(2.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    for (preset, label) in presets {
                                        let is_sel = state.filter_preset == preset;
                                        let seg_bg = if is_sel { Theme::ACCENT_DIM } else { Color32::TRANSPARENT };
                                        let seg_fg = if is_sel { Color32::WHITE } else { Theme::MUTED };
                                        let seg_stroke = if is_sel { Theme::stroke_accent_dim() } else { Stroke::NONE };

                                        let seg_galley = ui.painter().layout_no_wrap(
                                            label.to_string(),
                                            Theme::font_sans(12.0),
                                            seg_fg,
                                        );
                                        let seg_size = seg_galley.size() + Vec2::new(18.0, 8.0);
                                        let (seg_rect, seg_resp) = ui.allocate_exact_size(seg_size, egui::Sense::click());

                                        ui.painter().rect(
                                            seg_rect,
                                            Rounding::same(5.0),
                                            seg_bg,
                                            seg_stroke,
                                        );
                                        let seg_pos = seg_rect.center() - seg_galley.size() / 2.0;
                                        ui.painter().galley(seg_pos, seg_galley, seg_fg);

                                        if seg_resp.clicked() {
                                            state.filter_preset = preset;
                                            if preset == DateFilterPresetWrapper::Custom && state.custom_date_from.is_empty() {
                                                let first_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
                                                state.custom_date_from = Theme::format_input_date(&first_of_month);
                                                state.custom_date_to = Theme::format_input_date(&today);
                                            }
                                        }
                                    }
                                });
                            });

                        if state.filter_preset == DateFilterPresetWrapper::Custom {
                            ui.add_space(4.0);
                            ui.add(egui::TextEdit::singleline(&mut state.custom_date_from).desired_width(75.0).hint_text("M/D/YYYY"));
                            ui.label(RichText::new("-").color(Theme::MUTED));
                            ui.add(egui::TextEdit::singleline(&mut state.custom_date_to).desired_width(75.0).hint_text("M/D/YYYY"));
                        }

                        ui.add_space(10.0);

                        // Category Dropdown
                        egui::ComboBox::from_id_salt("cat_filter_dropdown")
                            .selected_text(if state.selected_category.is_empty() { "All categories" } else { &state.selected_category })
                            .show_ui(ui, |ui| {
                                if ui.selectable_label(state.selected_category.is_empty(), "All categories").clicked() {
                                    state.selected_category.clear();
                                }
                                for cat in &data.custom_categories {
                                    if ui.selectable_label(state.selected_category == *cat, cat).clicked() {
                                        state.selected_category = cat.clone();
                                    }
                                }
                            });

                        ui.add_space(8.0);

                        // Quick action: Sort same-date transactions with Spending first
                        let sort_btn = egui::Frame::none()
                            .stroke(Theme::stroke_border())
                            .rounding(Rounding::same(Theme::RADIUS_BTN))
                            .inner_margin(egui::Margin::symmetric(10.0, 5.0));
                        let sort_resp = sort_btn.show(ui, |ui| {
                            ui.label(RichText::new("⇅ Spending first").font(Theme::font_sans(12.0)).color(Theme::MUTED));
                        }).response.interact(egui::Sense::click()).on_hover_text("Sort same-date transactions so spendings appear before income");
                        if sort_resp.clicked() {
                            if sort_spending_first_on_same_date(data, current_card.id) {
                                *data_changed = true;
                            }
                        }

                        // Search box aligned to the right
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut state.search_query)
                                    .desired_width(180.0)
                                    .hint_text("Search transactions…"),
                            );
                        });
                    });
                });

            // 1px Border separator
            let (sep_rect2, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
            ui.painter().line_segment([sep_rect2.left_top(), sep_rect2.right_top()], Theme::stroke_border());

            // C. TRANSACTION TABLE SECTION (Spreadsheet Grid with visible cell grid lines and inline editing)
            let mut tx_to_delete = None;
            let mut tx_to_edit = None;
            let mut tx_to_swap: Option<(Uuid, Uuid)> = None;

            let max_expense_visible = displayed_rows
                .iter()
                .filter_map(|r| if !r.is_opening_balance { r.money_out } else { None })
                .fold(0.0_f64, f64::max);

            let editable_tx_ids: Vec<Uuid> = displayed_rows
                .iter()
                .filter_map(|r| r.transaction_id)
                .collect();

            TableBuilder::new(ui)
                .id_salt("accounts_spreadsheet_grid")
                .striped(false)
                .resizable(true)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::initial(105.0).resizable(true).at_least(85.0))   // Date
                .column(Column::initial(220.0).resizable(true).at_least(130.0))  // Description
                .column(Column::initial(130.0).resizable(true).at_least(90.0))   // Category
                .column(Column::initial(120.0).resizable(true).at_least(85.0))   // In (PKR)
                .column(Column::initial(120.0).resizable(true).at_least(85.0))   // Out (PKR)
                .column(Column::initial(130.0).resizable(true).at_least(90.0))   // Balance
                .column(Column::initial(130.0).resizable(true).at_least(110.0))  // Actions
                .header(34.0, |mut header| {
                    let draw_header_cell_border = |ui: &Ui, is_first: bool| {
                        let r = ui.max_rect();
                        ui.painter().line_segment([r.right_top(), r.right_bottom()], Theme::stroke_border());
                        ui.painter().line_segment([r.left_bottom(), r.right_bottom()], Theme::stroke_border());
                        if is_first {
                            ui.painter().line_segment([r.left_top(), r.left_bottom()], Theme::stroke_border());
                        }
                    };

                    header.col(|ui| {
                        draw_header_cell_border(ui, true);
                        ui.add_space(14.0);
                        ui.label(RichText::new("Date").font(Theme::font_sans(12.0)).color(Theme::MUTED));
                    });
                    header.col(|ui| {
                        draw_header_cell_border(ui, false);
                        ui.add_space(10.0);
                        ui.label(RichText::new("Description").font(Theme::font_sans(12.0)).color(Theme::MUTED));
                    });
                    header.col(|ui| {
                        draw_header_cell_border(ui, false);
                        ui.add_space(10.0);
                        ui.label(RichText::new("Category").font(Theme::font_sans(12.0)).color(Theme::MUTED));
                    });
                    header.col(|ui| {
                        draw_header_cell_border(ui, false);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_space(14.0);
                            ui.label(RichText::new("In (PKR)").font(Theme::font_sans(12.0)).color(Theme::MUTED));
                        });
                    });
                    header.col(|ui| {
                        draw_header_cell_border(ui, false);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_space(14.0);
                            ui.label(RichText::new("Out (PKR)").font(Theme::font_sans(12.0)).color(Theme::MUTED));
                        });
                    });
                    header.col(|ui| {
                        draw_header_cell_border(ui, false);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_space(14.0);
                            ui.label(RichText::new("Balance").font(Theme::font_sans(12.0)).color(Theme::MUTED));
                        });
                    });
                    header.col(|ui| {
                        draw_header_cell_border(ui, false);
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.label(RichText::new("Actions").font(Theme::font_sans(12.0)).color(Theme::MUTED));
                        });
                    });
                })
                .body(|mut body| {
                    for (row_idx, row) in displayed_rows.iter().enumerate() {
                        body.row(36.0, |mut r| {
                            let is_op = row.is_opening_balance;
                            let current_tx_id = row.transaction_id;

                            let draw_cell_border = |ui: &Ui, is_first: bool| {
                                let rect = ui.max_rect();
                                ui.painter().line_segment([rect.right_top(), rect.right_bottom()], Theme::stroke_border());
                                ui.painter().line_segment([rect.left_bottom(), rect.right_bottom()], Theme::stroke_border());
                                if is_first {
                                    ui.painter().line_segment([rect.left_top(), rect.left_bottom()], Theme::stroke_border());
                                }
                            };

                            // 1. Date Column
                            r.col(|ui| {
                                draw_cell_border(ui, true);
                                ui.add_space(14.0);
                                if is_op {
                                    ui.label(RichText::new(Theme::format_date(&row.date)).font(Theme::font_sans(12.5)).italics().color(Theme::FAINT));
                                } else {
                                    ui.label(RichText::new(Theme::format_date(&row.date)).font(Theme::font_sans(12.5)).color(Theme::MUTED));
                                }
                            });

                            // 2. Description Column (Inline Editable)
                            r.col(|ui| {
                                draw_cell_border(ui, false);
                                let cell_rect = ui.max_rect();

                                if is_op {
                                    ui.add_space(10.0);
                                    let resp = ui.interact(cell_rect, ui.id().with("op_desc"), egui::Sense::click());
                                    if resp.double_clicked() {
                                        state.open_edit_opening_balance(&current_card, today);
                                    }
                                    ui.label(RichText::new(&row.description).font(Theme::font_sans(13.0)).italics().color(Theme::FAINT));
                                } else if let Some(tx_id) = current_tx_id {
                                    let is_editing = state.active_edit == Some((tx_id, SpreadsheetCol::Description));

                                    if is_editing {
                                        let te = egui::TextEdit::singleline(&mut state.edit_str)
                                            .desired_width(ui.available_width() - 8.0);
                                        let te_resp = ui.add(te);
                                        if state.edit_just_focused {
                                            te_resp.request_focus();
                                            state.edit_just_focused = false;
                                        }

                                        handle_cell_keys(
                                            ui,
                                            &te_resp,
                                            data,
                                            state,
                                            tx_id,
                                            SpreadsheetCol::Description,
                                            &editable_tx_ids,
                                            data_changed,
                                        );
                                    } else {
                                        let resp = ui.interact(cell_rect, ui.id().with(tx_id).with(0), egui::Sense::click());
                                        if resp.double_clicked() {
                                            state.active_edit = Some((tx_id, SpreadsheetCol::Description));
                                            state.edit_str = load_cell_val(&data.transactions, tx_id, SpreadsheetCol::Description);
                                            state.edit_just_focused = true;
                                        }
                                        ui.add_space(10.0);
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&row.description).font(Theme::font_sans(13.0)).color(Theme::TEXT));
                                            if row.auto_generated {
                                                ui.colored_label(Theme::ACCENT, RichText::new("[Auto]").font(Theme::font_sans(10.0)));
                                            }
                                        });
                                    }
                                }
                            });

                            // 3. Category Column (Inline Chip / Dropdown Editable)
                            r.col(|ui| {
                                draw_cell_border(ui, false);
                                let cell_rect = ui.max_rect();

                                if is_op {
                                    ui.add_space(10.0);
                                    ui.label(RichText::new("—").font(Theme::font_sans(12.5)).italics().color(Theme::FAINT));
                                } else if let Some(tx_id) = current_tx_id {
                                    let is_editing = state.active_edit == Some((tx_id, SpreadsheetCol::Category));

                                    if is_editing {
                                        let mut selected_cat = state.edit_str.clone();
                                        egui::ComboBox::from_id_salt(format!("cat_combo_{}", tx_id))
                                            .width(ui.available_width() - 8.0)
                                            .selected_text(&selected_cat)
                                            .show_ui(ui, |ui| {
                                                for cat in &data.custom_categories {
                                                    if ui.selectable_label(selected_cat == *cat, cat).clicked() {
                                                        selected_cat = cat.clone();
                                                        state.edit_str = cat.clone();
                                                        commit_cell(&mut data.transactions, tx_id, SpreadsheetCol::Category, &state.edit_str, data_changed);
                                                        state.active_edit = None;
                                                    }
                                                }
                                            });

                                        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                                        let tab_pressed = ui.input(|i| i.key_pressed(egui::Key::Tab));
                                        let esc_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));
                                        if esc_pressed {
                                            state.active_edit = None;
                                        } else if enter_pressed || tab_pressed {
                                            commit_cell(&mut data.transactions, tx_id, SpreadsheetCol::Category, &state.edit_str, data_changed);
                                            state.active_edit = None;
                                        }
                                    } else {
                                        let resp = ui.interact(cell_rect, ui.id().with(tx_id).with(1), egui::Sense::click());
                                        if resp.double_clicked() {
                                            state.active_edit = Some((tx_id, SpreadsheetCol::Category));
                                            state.edit_str = load_cell_val(&data.transactions, tx_id, SpreadsheetCol::Category);
                                            state.edit_just_focused = true;
                                        }

                                        // Render category color chip per Section 7.3
                                        let cat_color = Theme::category_color(&row.category);
                                        let bg_chip = Color32::from_rgba_premultiplied(
                                            (cat_color.r() as f32 * 0.15) as u8,
                                            (cat_color.g() as f32 * 0.15) as u8,
                                            (cat_color.b() as f32 * 0.15) as u8,
                                            38,
                                        );

                                        let galley = ui.painter().layout_no_wrap(
                                            row.category.clone(),
                                            Theme::font_sans(12.0),
                                            cat_color,
                                        );
                                        let chip_w = (galley.size().x + 14.0).min(cell_rect.width() - 16.0);
                                        let chip_h = 24.0;
                                        let chip_rect = egui::Rect::from_min_size(
                                            pos2(cell_rect.left() + 8.0, cell_rect.center().y - chip_h / 2.0),
                                            vec2(chip_w, chip_h),
                                        );

                                        ui.painter().rect_filled(chip_rect, Rounding::same(Theme::RADIUS_BADGE), bg_chip);
                                        ui.painter().galley(chip_rect.center() - galley.size() / 2.0, galley, cat_color);
                                    }
                                }
                            });

                            // 4. In (PKR) Column (Inline Editable)
                            r.col(|ui| {
                                draw_cell_border(ui, false);
                                let cell_rect = ui.max_rect();

                                if is_op {
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.add_space(14.0);
                                        ui.label(RichText::new("—").font(Theme::font_mono(13.0)).color(Theme::FAINT));
                                    });
                                } else if let Some(tx_id) = current_tx_id {
                                    let is_editing = state.active_edit == Some((tx_id, SpreadsheetCol::MoneyIn));

                                    if is_editing {
                                        let te = egui::TextEdit::singleline(&mut state.edit_str)
                                            .desired_width(ui.available_width() - 8.0);
                                        let te_resp = ui.add(te);
                                        if state.edit_just_focused {
                                            te_resp.request_focus();
                                            state.edit_just_focused = false;
                                        }

                                        handle_cell_keys(
                                            ui,
                                            &te_resp,
                                            data,
                                            state,
                                            tx_id,
                                            SpreadsheetCol::MoneyIn,
                                            &editable_tx_ids,
                                            data_changed,
                                        );
                                    } else {
                                        let resp = ui.interact(cell_rect, ui.id().with(tx_id).with(2), egui::Sense::click());
                                        if resp.double_clicked() {
                                            state.active_edit = Some((tx_id, SpreadsheetCol::MoneyIn));
                                            state.edit_str = load_cell_val(&data.transactions, tx_id, SpreadsheetCol::MoneyIn);
                                            state.edit_just_focused = true;
                                        }

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            ui.add_space(14.0);
                                            if let Some(val) = row.money_in {
                                                Theme::money_label(ui, &Theme::format_pkr(val), Theme::IN, 13.0);
                                            } else {
                                                ui.label(RichText::new("—").font(Theme::font_mono(13.0)).color(Theme::FAINT));
                                            }
                                        });
                                    }
                                }
                            });

                            // 5. Out (PKR) Column (Data Bar Overlay + Inline Editable)
                            r.col(|ui| {
                                draw_cell_border(ui, false);
                                let cell_rect = ui.max_rect();

                                if is_op {
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.add_space(14.0);
                                        ui.label(RichText::new("—").font(Theme::font_mono(13.0)).color(Theme::FAINT));
                                    });
                                } else if let Some(tx_id) = current_tx_id {
                                    let is_editing = state.active_edit == Some((tx_id, SpreadsheetCol::MoneyOut));

                                    if is_editing {
                                        let te = egui::TextEdit::singleline(&mut state.edit_str)
                                            .desired_width(ui.available_width() - 8.0);
                                        let te_resp = ui.add(te);
                                        if state.edit_just_focused {
                                            te_resp.request_focus();
                                            state.edit_just_focused = false;
                                        }

                                        handle_cell_keys(
                                            ui,
                                            &te_resp,
                                            data,
                                            state,
                                            tx_id,
                                            SpreadsheetCol::MoneyOut,
                                            &editable_tx_ids,
                                            data_changed,
                                        );
                                    } else {
                                        let resp = ui.interact(cell_rect, ui.id().with(tx_id).with(3), egui::Sense::click());
                                        if resp.double_clicked() {
                                            state.active_edit = Some((tx_id, SpreadsheetCol::MoneyOut));
                                            state.edit_str = load_cell_val(&data.transactions, tx_id, SpreadsheetCol::MoneyOut);
                                            state.edit_just_focused = true;
                                        }

                                        // Render inline data-bar overlay behind numeric text
                                        if let Some(val) = row.money_out {
                                            if max_expense_visible > 0.0 {
                                                let ratio = (val / max_expense_visible).clamp(0.0, 1.0) as f32;
                                                let bar_w = ((cell_rect.width() - 8.0) * ratio).max(3.0);
                                                let bar_rect = egui::Rect::from_min_size(
                                                    pos2(cell_rect.left() + 4.0, cell_rect.bottom() - 3.5),
                                                    vec2(bar_w, 3.0),
                                                );
                                                // --out at 25% opacity
                                                let bar_col = Color32::from_rgba_premultiplied(60, 28, 26, 64);
                                                ui.painter().rect_filled(bar_rect, Rounding::same(1.5), bar_col);
                                            }
                                        }

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            ui.add_space(14.0);
                                            if let Some(val) = row.money_out {
                                                Theme::money_label(ui, &Theme::format_pkr(val), Theme::OUT, 13.0);
                                            } else {
                                                ui.label(RichText::new("—").font(Theme::font_mono(13.0)).color(Theme::FAINT));
                                            }
                                        });
                                    }
                                }
                            });

                            // 6. Balance Column
                            r.col(|ui| {
                                draw_cell_border(ui, false);
                                let cell_rect = ui.max_rect();
                                if is_op {
                                    let resp = ui.interact(cell_rect, ui.id().with("op_bal"), egui::Sense::click());
                                    if resp.double_clicked() {
                                        state.open_edit_opening_balance(&current_card, today);
                                    }
                                }

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.add_space(14.0);
                                    let bal_col = if is_op {
                                        Theme::TEXT
                                    } else if row.running_balance >= 0.0 {
                                        Theme::TEXT
                                    } else {
                                        Theme::OUT
                                    };
                                    Theme::money_label(ui, &Theme::format_pkr(row.running_balance), bal_col, 13.0);
                                });
                            });

                            // 7. Actions Column
                            r.col(|ui| {
                                draw_cell_border(ui, false);
                                ui.horizontal(|ui| {
                                    ui.add_space(4.0);
                                    if is_op {
                                        if ui.link(RichText::new("Edit").font(Theme::font_sans(12.0)).color(Theme::FAINT)).clicked() {
                                            state.open_edit_opening_balance(&current_card, today);
                                        }
                                    } else if let Some(tx_id) = row.transaction_id {
                                        let row_date = row.date;
                                        let can_move_up = row_idx > 0
                                            && displayed_rows[row_idx - 1].transaction_id.is_some()
                                            && displayed_rows[row_idx - 1].date == row_date;
                                        let prev_tx_id = if can_move_up { displayed_rows[row_idx - 1].transaction_id } else { None };

                                        let can_move_down = row_idx + 1 < displayed_rows.len()
                                            && displayed_rows[row_idx + 1].transaction_id.is_some()
                                            && displayed_rows[row_idx + 1].date == row_date;
                                        let next_tx_id = if can_move_down { displayed_rows[row_idx + 1].transaction_id } else { None };

                                        if can_move_up {
                                            if ui.small_button(RichText::new("▲").size(10.0).color(Theme::TEXT)).on_hover_text("Move up on same date").clicked() {
                                                if let Some(other_id) = prev_tx_id {
                                                    tx_to_swap = Some((tx_id, other_id));
                                                }
                                            }
                                        } else {
                                            ui.add_enabled(false, egui::Button::new(RichText::new("▲").size(10.0)).small());
                                        }

                                        if can_move_down {
                                            if ui.small_button(RichText::new("▼").size(10.0).color(Theme::TEXT)).on_hover_text("Move down on same date").clicked() {
                                                if let Some(other_id) = next_tx_id {
                                                    tx_to_swap = Some((tx_id, other_id));
                                                }
                                            }
                                        } else {
                                            ui.add_enabled(false, egui::Button::new(RichText::new("▼").size(10.0)).small());
                                        }

                                        ui.add_space(2.0);
                                        if ui.link(RichText::new("Edit").font(Theme::font_sans(12.0)).color(Theme::FAINT)).clicked() {
                                            if let Some(tx) = data.transactions.iter().find(|t| t.id == tx_id) {
                                                tx_to_edit = Some(tx.clone());
                                            }
                                        }
                                        ui.add_space(4.0);
                                        if ui.link(RichText::new("Del").font(Theme::font_sans(12.0)).color(Theme::FAINT)).clicked() {
                                            tx_to_delete = Some(tx_id);
                                        }
                                    }
                                });
                            });
                        });
                    }
                });

            if let Some((id1, id2)) = tx_to_swap {
                if let (Some(idx1), Some(idx2)) = (
                    data.transactions.iter().position(|t| t.id == id1),
                    data.transactions.iter().position(|t| t.id == id2),
                ) {
                    data.transactions.swap(idx1, idx2);
                    let now = chrono::Utc::now();
                    data.transactions[idx1].updated_at = now;
                    data.transactions[idx2].updated_at = now;
                    *data_changed = true;
                }
            }

            if let Some(tx) = tx_to_edit {
                state.open_edit_tx(&tx);
            }

            if let Some(del_id) = tx_to_delete {
                let now = chrono::Utc::now();
                if let Some(t) = data.transactions.iter_mut().find(|t| t.id == del_id) {
                    t.deleted_at = Some(now);
                    t.updated_at = now;
                }
                *data_changed = true;
            }
        });
    });

    // --- MODALS ---

    // 1. Add Card Modal
    if state.show_add_card {
        egui::Window::new("Add New Card / Account")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(320.0);
                ui.add_space(8.0);

                ui.label("Card / Account Name:");
                ui.text_edit_singleline(&mut state.new_card_name);

                ui.add_space(8.0);
                ui.label("Opening Balance (PKR):");
                ui.text_edit_singleline(&mut state.new_card_opening_balance);

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Create Card").strong().color(Theme::ACCENT)).clicked() {
                        let name = state.new_card_name.trim().to_string();
                        if !name.is_empty() {
                            let op_bal = Theme::parse_pkr_input(&state.new_card_opening_balance).unwrap_or(0.0);
                            let now = chrono::Utc::now();
                            let new_card = Card {
                                id: Uuid::new_v4(),
                                name,
                                is_primary: data.cards.iter().all(|c| c.deleted_at.is_some()),
                                opening_balance: op_bal,
                                opening_balance_description: "Opening Balance".to_string(),
                                opening_balance_date: None,
                                updated_at: now,
                                deleted_at: None,
                            };
                            let new_id = new_card.id;
                            data.cards.push(new_card);
                            state.selected_card_id = Some(new_id);
                            *data_changed = true;
                            state.show_add_card = false;
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        state.show_add_card = false;
                    }
                });
            });
    }

    // 2. Rename Card Modal
    if state.show_rename_card {
        egui::Window::new("Rename Card")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(300.0);
                ui.label("New Card Name:");
                ui.text_edit_singleline(&mut state.rename_card_name);

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Save").color(Theme::ACCENT)).clicked() {
                        let name = state.rename_card_name.trim().to_string();
                        if !name.is_empty() {
                            if let Some(c) = data.cards.iter_mut().find(|c| c.id == active_card_id) {
                                c.name = name;
                                c.updated_at = chrono::Utc::now();
                                *data_changed = true;
                            }
                            state.show_rename_card = false;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_rename_card = false;
                    }
                });
            });
    }

    // 3. Delete Card Confirmation Modal
    if let Some(del_card_id) = state.card_to_delete {
        egui::Window::new("Confirm Delete Card")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(320.0);
                ui.label("Are you sure you want to delete this card and its transactions?");
                ui.label(RichText::new("This action cannot be undone.").color(Theme::EXPENSE).small());

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Delete Permanently").color(Theme::EXPENSE).strong()).clicked() {
                        let now = chrono::Utc::now();
                        if let Some(c) = data.cards.iter_mut().find(|c| c.id == del_card_id) {
                            c.deleted_at = Some(now);
                            c.updated_at = now;
                        }
                        for t in &mut data.transactions {
                            if t.card_id == del_card_id {
                                t.deleted_at = Some(now);
                                t.updated_at = now;
                            }
                        }
                        if let Some(first) = data.cards.iter().find(|c| c.deleted_at.is_none()) {
                            let first_id = first.id;
                            for s in &mut data.subscriptions {
                                if s.card_id == del_card_id {
                                    s.card_id = first_id;
                                    s.updated_at = now;
                                }
                            }
                        }
                        if state.selected_card_id == Some(del_card_id) {
                            state.selected_card_id = data.cards.iter().find(|c| c.deleted_at.is_none()).map(|c| c.id);
                        }
                        *data_changed = true;
                        state.card_to_delete = None;
                    }
                    if ui.button("Cancel").clicked() {
                        state.card_to_delete = None;
                    }
                });
            });
    }

    // 4. Add / Edit Transaction Modal
    if state.show_tx_modal {
        let is_edit = state.editing_tx_id.is_some();
        let title = if is_edit { "Edit Transaction" } else { "Add Transaction" };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(360.0);
                ui.add_space(6.0);

                // Income / Expense Toggle
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    if ui.selectable_label(state.tx_is_income, RichText::new("Income (In)").color(Theme::INCOME).strong()).clicked() {
                        state.tx_is_income = true;
                        if state.tx_category == "Food" || state.tx_category.is_empty() {
                            state.tx_category = "Income".to_string();
                        }
                    }
                    if ui.selectable_label(!state.tx_is_income, RichText::new("Expense (Out)").color(Theme::EXPENSE).strong()).clicked() {
                        state.tx_is_income = false;
                        if state.tx_category == "Income" {
                            state.tx_category = "Food".to_string();
                        }
                    }
                });

                ui.add_space(8.0);
                ui.label("Date (M/D/YYYY):");
                ui.text_edit_singleline(&mut state.tx_date);

                ui.add_space(6.0);
                ui.label("Description:");
                ui.text_edit_singleline(&mut state.tx_description);

                ui.add_space(6.0);
                ui.label("Category:");
                if state.tx_is_income {
                    ui.horizontal_wrapped(|ui| {
                        let chips = ["Income", "Salary", "Freelance", "Pocket Money", "Money Received"];
                        for chip in chips {
                            if ui.selectable_label(state.tx_category == chip, chip).clicked() {
                                state.tx_category = chip.to_string();
                            }
                        }
                    });
                } else {
                    egui::ComboBox::from_id_salt("tx_cat_select")
                        .selected_text(&state.tx_category)
                        .show_ui(ui, |ui| {
                            for cat in &data.custom_categories {
                                if cat != "Income" {
                                    if ui.selectable_label(state.tx_category == *cat, cat).clicked() {
                                        state.tx_category = cat.clone();
                                    }
                                }
                            }
                        });
                }

                ui.horizontal(|ui| {
                    ui.label("Custom Category:");
                    ui.text_edit_singleline(&mut state.tx_category);
                });

                ui.add_space(6.0);
                ui.label("Amount (PKR):");
                ui.text_edit_singleline(&mut state.tx_amount);

                if let Some(err) = &state.tx_error {
                    ui.colored_label(Theme::EXPENSE, err);
                }

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Save Transaction").strong().color(Theme::ACCENT)).clicked() {
                        let date = match Theme::parse_date_input(&state.tx_date) {
                            Some(d) => d,
                            None => {
                                state.tx_error = Some("Invalid date format. Use M/D/YYYY (e.g. 9/4/2026).".to_string());
                                return;
                            }
                        };

                        let desc = state.tx_description.trim().to_string();
                        if desc.is_empty() {
                            state.tx_error = Some("Description cannot be empty.".to_string());
                            return;
                        }

                        let amount = match Theme::parse_pkr_input(&state.tx_amount) {
                            Some(a) if a > 0.0 => a,
                            _ => {
                                state.tx_error = Some("Amount must be a positive number.".to_string());
                                return;
                            }
                        };

                        let category = if state.tx_category.trim().is_empty() {
                            if state.tx_is_income { "Income".to_string() } else { "Other".to_string() }
                        } else {
                            state.tx_category.trim().to_string()
                        };

                        if !data.custom_categories.iter().any(|c| c.eq_ignore_ascii_case(&category)) {
                            data.custom_categories.push(category.clone());
                        }

                        let now = chrono::Utc::now();
                        if let Some(edit_id) = state.editing_tx_id {
                            if let Some(tx) = data.transactions.iter_mut().find(|t| t.id == edit_id) {
                                tx.date = date;
                                tx.description = desc;
                                tx.category = category;
                                tx.amount = amount;
                                tx.is_income = state.tx_is_income;
                                tx.updated_at = now;
                            }
                        } else {
                            let new_tx = Transaction {
                                id: Uuid::new_v4(),
                                card_id: active_card_id,
                                date,
                                description: desc,
                                category,
                                amount,
                                is_income: state.tx_is_income,
                                auto_generated: false,
                                updated_at: now,
                                deleted_at: None,
                            };
                            data.transactions.push(new_tx);
                        }

                        *data_changed = true;
                        state.show_tx_modal = false;
                    }

                    if ui.button("Cancel").clicked() {
                        state.show_tx_modal = false;
                    }
                });
            });
    }

    // 5. Edit Opening Balance Modal
    if state.show_opening_balance_modal {
        egui::Window::new("Edit Opening Balance")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(340.0);
                ui.add_space(8.0);

                ui.label("Description:");
                ui.text_edit_singleline(&mut state.op_bal_description);

                ui.add_space(6.0);
                ui.label("Date (M/D/YYYY):");
                ui.text_edit_singleline(&mut state.op_bal_date);

                ui.add_space(6.0);
                ui.label("Amount (PKR):");
                ui.text_edit_singleline(&mut state.op_bal_amount);

                if let Some(err) = &state.op_bal_error {
                    ui.colored_label(Theme::EXPENSE, err);
                }

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Save").strong().color(Theme::ACCENT)).clicked() {
                        let date = match Theme::parse_date_input(&state.op_bal_date) {
                            Some(d) => d,
                            None => {
                                state.op_bal_error = Some("Invalid date format. Use M/D/YYYY (e.g. 9/4/2026).".to_string());
                                return;
                            }
                        };

                        let desc = state.op_bal_description.trim().to_string();
                        let desc = if desc.is_empty() {
                            "Opening Balance".to_string()
                        } else {
                            desc
                        };

                        let amount = match Theme::parse_pkr_input(&state.op_bal_amount) {
                            Some(a) => a,
                            None => {
                                state.op_bal_error = Some("Invalid amount.".to_string());
                                return;
                            }
                        };

                        if let Some(card) = data.cards.iter_mut().find(|c| c.id == active_card_id) {
                            card.opening_balance = amount;
                            card.opening_balance_description = desc;
                            card.opening_balance_date = Some(date);
                            card.updated_at = chrono::Utc::now();
                            *data_changed = true;
                        }
                        state.show_opening_balance_modal = false;
                    }

                    if ui.button("Cancel").clicked() {
                        state.show_opening_balance_modal = false;
                    }
                });
            });
    }
}

fn load_cell_val(txs: &[Transaction], tx_id: Uuid, col: SpreadsheetCol) -> String {
    if let Some(tx) = txs.iter().find(|t| t.id == tx_id) {
        match col {
            SpreadsheetCol::Description => tx.description.clone(),
            SpreadsheetCol::Category => tx.category.clone(),
            SpreadsheetCol::MoneyIn => {
                if tx.is_income {
                    format!("{:.2}", tx.amount)
                } else {
                    String::new()
                }
            }
            SpreadsheetCol::MoneyOut => {
                if !tx.is_income {
                    format!("{:.2}", tx.amount)
                } else {
                    String::new()
                }
            }
        }
    } else {
        String::new()
    }
}

fn commit_cell(
    txs: &mut [Transaction],
    tx_id: Uuid,
    col: SpreadsheetCol,
    val: &str,
    data_changed: &mut bool,
) {
    if let Some(tx) = txs.iter_mut().find(|t| t.id == tx_id) {
        let trimmed = val.trim();
        match col {
            SpreadsheetCol::Description => {
                if !trimmed.is_empty() && tx.description != trimmed {
                    tx.description = trimmed.to_string();
                    tx.updated_at = chrono::Utc::now();
                    *data_changed = true;
                }
            }
            SpreadsheetCol::Category => {
                if !trimmed.is_empty() && tx.category != trimmed {
                    tx.category = trimmed.to_string();
                    tx.updated_at = chrono::Utc::now();
                    *data_changed = true;
                }
            }
            SpreadsheetCol::MoneyIn => {
                if let Some(parsed) = Theme::parse_pkr_input(trimmed) {
                    if parsed > 0.0 {
                        tx.amount = parsed;
                        tx.is_income = true;
                        if tx.category == "Other" || tx.category.is_empty() {
                            tx.category = "Income".to_string();
                        }
                        tx.updated_at = chrono::Utc::now();
                        *data_changed = true;
                    }
                }
            }
            SpreadsheetCol::MoneyOut => {
                if let Some(parsed) = Theme::parse_pkr_input(trimmed) {
                    if parsed > 0.0 {
                        tx.amount = parsed;
                        tx.is_income = false;
                        if tx.category == "Income" {
                            tx.category = "Other".to_string();
                        }
                        tx.updated_at = chrono::Utc::now();
                        *data_changed = true;
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_cell_keys(
    ui: &Ui,
    resp: &egui::Response,
    data: &mut AppData,
    state: &mut AccountsViewState,
    tx_id: Uuid,
    col: SpreadsheetCol,
    editable_tx_ids: &[Uuid],
    data_changed: &mut bool,
) {
    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
    let tab_pressed = ui.input(|i| i.key_pressed(egui::Key::Tab));
    let shift_held = ui.input(|i| i.modifiers.shift);
    let esc_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));

    if esc_pressed {
        state.active_edit = None;
    } else if enter_pressed {
        // Commit and move down one row in same column
        commit_cell(&mut data.transactions, tx_id, col, &state.edit_str, data_changed);
        if let Some(pos) = editable_tx_ids.iter().position(|id| *id == tx_id) {
            if pos + 1 < editable_tx_ids.len() {
                let next_id = editable_tx_ids[pos + 1];
                state.active_edit = Some((next_id, col));
                state.edit_str = load_cell_val(&data.transactions, next_id, col);
                state.edit_just_focused = true;
            } else {
                state.active_edit = None;
            }
        } else {
            state.active_edit = None;
        }
    } else if tab_pressed {
        commit_cell(&mut data.transactions, tx_id, col, &state.edit_str, data_changed);
        if shift_held {
            match col {
                SpreadsheetCol::MoneyOut => {
                    state.active_edit = Some((tx_id, SpreadsheetCol::MoneyIn));
                    state.edit_str = load_cell_val(&data.transactions, tx_id, SpreadsheetCol::MoneyIn);
                    state.edit_just_focused = true;
                }
                SpreadsheetCol::MoneyIn => {
                    state.active_edit = Some((tx_id, SpreadsheetCol::Category));
                    state.edit_str = load_cell_val(&data.transactions, tx_id, SpreadsheetCol::Category);
                    state.edit_just_focused = true;
                }
                SpreadsheetCol::Category => {
                    state.active_edit = Some((tx_id, SpreadsheetCol::Description));
                    state.edit_str = load_cell_val(&data.transactions, tx_id, SpreadsheetCol::Description);
                    state.edit_just_focused = true;
                }
                SpreadsheetCol::Description => {
                    if let Some(pos) = editable_tx_ids.iter().position(|id| *id == tx_id) {
                        if pos > 0 {
                            let prev_id = editable_tx_ids[pos - 1];
                            state.active_edit = Some((prev_id, SpreadsheetCol::MoneyOut));
                            state.edit_str = load_cell_val(&data.transactions, prev_id, SpreadsheetCol::MoneyOut);
                            state.edit_just_focused = true;
                        } else {
                            state.active_edit = None;
                        }
                    } else {
                        state.active_edit = None;
                    }
                }
            }
        } else {
            match col {
                SpreadsheetCol::Description => {
                    state.active_edit = Some((tx_id, SpreadsheetCol::Category));
                    state.edit_str = load_cell_val(&data.transactions, tx_id, SpreadsheetCol::Category);
                    state.edit_just_focused = true;
                }
                SpreadsheetCol::Category => {
                    state.active_edit = Some((tx_id, SpreadsheetCol::MoneyIn));
                    state.edit_str = load_cell_val(&data.transactions, tx_id, SpreadsheetCol::MoneyIn);
                    state.edit_just_focused = true;
                }
                SpreadsheetCol::MoneyIn => {
                    state.active_edit = Some((tx_id, SpreadsheetCol::MoneyOut));
                    state.edit_str = load_cell_val(&data.transactions, tx_id, SpreadsheetCol::MoneyOut);
                    state.edit_just_focused = true;
                }
                SpreadsheetCol::MoneyOut => {
                    if let Some(pos) = editable_tx_ids.iter().position(|id| *id == tx_id) {
                        if pos + 1 < editable_tx_ids.len() {
                            let next_id = editable_tx_ids[pos + 1];
                            state.active_edit = Some((next_id, SpreadsheetCol::Description));
                            state.edit_str = load_cell_val(&data.transactions, next_id, SpreadsheetCol::Description);
                            state.edit_just_focused = true;
                        } else {
                            state.active_edit = None;
                        }
                    } else {
                        state.active_edit = None;
                    }
                }
            }
        }
    } else if resp.lost_focus() && !state.edit_just_focused {
        commit_cell(&mut data.transactions, tx_id, col, &state.edit_str, data_changed);
        state.active_edit = None;
    }
}

/// Sorts transactions on the same date for a card so that spendings appear before incomes,
/// preserving relative order within spending and within income.
pub fn sort_spending_first_on_same_date(data: &mut AppData, card_id: Uuid) -> bool {
    let now = chrono::Utc::now();
    let mut changed = false;

    // Collect all transaction indices for this card (non-deleted)
    let card_tx_indices: Vec<usize> = data.transactions
        .iter()
        .enumerate()
        .filter(|(_, t)| t.card_id == card_id && t.deleted_at.is_none())
        .map(|(i, _)| i)
        .collect();

    // Group indices by date
    let mut date_groups: std::collections::BTreeMap<chrono::NaiveDate, Vec<usize>> = std::collections::BTreeMap::new();
    for idx in card_tx_indices {
        date_groups.entry(data.transactions[idx].date).or_default().push(idx);
    }

    for (_date, indices) in date_groups {
        if indices.len() <= 1 {
            continue;
        }

        // We want spending (!is_income) first, income (is_income) second, stable.
        let mut sorted_indices = indices.clone();
        sorted_indices.sort_by_key(|&idx| {
            if data.transactions[idx].is_income { 1 } else { 0 }
        });

        if sorted_indices != indices {
            changed = true;
            let items: Vec<Transaction> = sorted_indices.iter().map(|&i| {
                let mut tx = data.transactions[i].clone();
                tx.updated_at = now;
                tx
            }).collect();

            for (slot, item) in indices.iter().zip(items.into_iter()) {
                data.transactions[*slot] = item;
            }
        }
    }

    changed
}

