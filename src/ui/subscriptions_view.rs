use crate::models::{AppData, BillingCycle, Subscription};
use crate::subscriptions::SubscriptionManager;
use crate::ui::theme::Theme;
use chrono::{Local, NaiveDate};
use egui::{Align, Layout, RichText, Rounding, Ui, Vec2};
use uuid::Uuid;

#[derive(Default)]
pub struct SubscriptionsViewState {
    pub show_modal: bool,
    pub editing_sub_id: Option<Uuid>,
    pub name: String,
    pub amount: String,
    pub cycle_type: usize, // 0 = Monthly, 1 = Yearly, 2 = Custom
    pub custom_days: String,
    pub start_date: String,
    pub linked_card_id: Option<Uuid>,
    pub modal_error: Option<String>,

    pub sub_to_delete: Option<Uuid>,
}

impl SubscriptionsViewState {
    pub fn open_add(&mut self, default_card_id: Option<Uuid>) {
        self.show_modal = true;
        self.editing_sub_id = None;
        self.name.clear();
        self.amount.clear();
        self.cycle_type = 0;
        self.custom_days = "30".to_string();
        self.start_date = Local::now().date_naive().format("%Y-%m-%d").to_string();
        self.linked_card_id = default_card_id;
        self.modal_error = None;
    }

    pub fn open_edit(&mut self, sub: &Subscription) {
        self.show_modal = true;
        self.editing_sub_id = Some(sub.id);
        self.name = sub.name.clone();
        self.amount = format!("{:.2}", sub.amount);
        match sub.cycle {
            BillingCycle::Monthly => {
                self.cycle_type = 0;
                self.custom_days = "30".to_string();
            }
            BillingCycle::Yearly => {
                self.cycle_type = 1;
                self.custom_days = "365".to_string();
            }
            BillingCycle::CustomDays(days) => {
                self.cycle_type = 2;
                self.custom_days = days.to_string();
            }
        }
        self.start_date = sub.start_date.format("%Y-%m-%d").to_string();
        self.linked_card_id = Some(sub.card_id);
        self.modal_error = None;
    }
}

pub fn render_subscriptions_view(
    ui: &mut Ui,
    data: &mut AppData,
    state: &mut SubscriptionsViewState,
    data_changed: &mut bool,
) {
    let today = Local::now().date_naive();
    let total_monthly = SubscriptionManager::total_monthly_cost(&data.subscriptions, None);
    let total_yearly = total_monthly * 12.0;
    let active_count = data.subscriptions.iter().filter(|s| !s.paused).count();

    let avail_width = (ui.available_width() - 2.0 * Theme::PAGE_MARGIN).max(0.0);
    let content_width = avail_width.min(Theme::CONTENT_MAX_WIDTH);
    let side_margin = Theme::PAGE_MARGIN + (avail_width - content_width) / 2.0;

    ui.add_space(Theme::CARD_GAP);

    // Header summary row (centered, constrained to content_width)
    ui.horizontal(|ui| {
        ui.add_space(side_margin);
        ui.vertical(|ui| {
            ui.set_width(content_width);
            ui.set_max_width(content_width);

            egui::Frame::none()
                .fill(Theme::PANEL)
                .stroke(Theme::stroke_border())
                .rounding(Rounding::same(Theme::RADIUS_PANEL))
                .inner_margin(egui::Margin::symmetric(22.0, Theme::CARD_PADDING))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Total monthly cost (prorated)").font(Theme::font_sans(11.5)).color(Theme::MUTED));
                            Theme::render_mono(ui, &Theme::format_pkr(total_monthly), Theme::ACCENT, 16.0);
                        });

                        ui.add_space(32.0);

                        ui.vertical(|ui| {
                            ui.label(RichText::new("Annual commitment").font(Theme::font_sans(11.5)).color(Theme::MUTED));
                            Theme::render_mono(ui, &Theme::format_pkr(total_yearly), Theme::TEXT, 16.0);
                        });

                        ui.add_space(32.0);

                        ui.vertical(|ui| {
                            ui.label(RichText::new("Active recurring").font(Theme::font_sans(11.5)).color(Theme::MUTED));
                            ui.label(
                                RichText::new(format!("{} active ({} total)", active_count, data.subscriptions.len()))
                                    .font(Theme::font_sans(14.0))
                                    .color(Theme::TEXT),
                            );
                        });
                    });
                });
        });
    });

    ui.add_space(Theme::CARD_GAP);

    // --- SINGLE-COLUMN VERTICAL STACK OF SUBSCRIPTION CARDS ---
    egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(side_margin);
            ui.vertical(|ui| {
                ui.set_width(content_width);
                ui.set_max_width(content_width);

                let mut sub_to_edit = None;

                for sub in &mut data.subscriptions {
                    let card_frame = egui::Frame::none()
                        .fill(Theme::PANEL)
                        .stroke(Theme::stroke_border())
                        .rounding(Rounding::same(Theme::RADIUS_CARD))
                        .inner_margin(egui::Margin::symmetric(20.0, Theme::CARD_PADDING));

                    card_frame.show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        // Row 1: Name (.nm: Space Grotesk 15px bold) and Status
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&sub.name)
                                    .font(Theme::font_head(15.0))
                                    .strong()
                                    .color(Theme::TEXT),
                            );

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if sub.paused {
                                    ui.label(RichText::new("Paused").font(Theme::font_sans(11.0)).color(Theme::FAINT));
                                } else {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Active").font(Theme::font_sans(11.0)).color(Theme::IN));
                                        let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(6.0), egui::Sense::hover());
                                        ui.painter().circle_filled(dot_rect.center(), 2.5, Theme::IN);
                                    });
                                }
                            });
                        });

                        ui.add_space(6.0);

                        // Row 2: Amount (.amt: IBM Plex Mono 16px, color var(--out))
                        let cycle_suffix = match sub.cycle {
                            BillingCycle::Monthly => "/ mo",
                            BillingCycle::Yearly => "/ yr",
                            BillingCycle::CustomDays(d) => &format!("/ {}d", d),
                        };
                        let amount_display = format!("{} {}", Theme::format_pkr(sub.amount), cycle_suffix);
                        Theme::render_mono(ui, &amount_display, Theme::OUT, 16.0);

                        ui.add_space(6.0);

                        // Row 3: Muted meta line (.meta: Inter 12.5px, color var(--muted))
                        let card_name = data.cards.iter().find(|c| c.id == sub.card_id).map(|c| c.name.as_str()).unwrap_or("Main Card");
                        let days_until = (sub.next_due_date - today).num_days();
                        let due_str = if days_until <= 3 && !sub.paused {
                            format!("next due {} (in {}d)", Theme::format_date(&sub.next_due_date), days_until)
                        } else {
                            format!("next due {}", Theme::format_date(&sub.next_due_date))
                        };
                        let meta_text = format!("Bills {} · {}", card_name, due_str);

                        ui.label(
                            RichText::new(meta_text)
                                .font(Theme::font_sans(12.5))
                                .color(if days_until <= 3 && !sub.paused { Theme::OUT } else { Theme::MUTED }),
                        );

                        ui.add_space(6.0);

                        // Row 4: Actions bar
                        ui.horizontal(|ui| {
                            if ui.link(RichText::new("Edit").font(Theme::font_sans(12.0)).color(Theme::FAINT)).clicked() {
                                sub_to_edit = Some(sub.clone());
                            }

                            ui.add_space(12.0);

                            let pause_text = if sub.paused { "Resume" } else { "Pause" };
                            if ui.link(RichText::new(pause_text).font(Theme::font_sans(12.0)).color(Theme::FAINT)).clicked() {
                                sub.paused = !sub.paused;
                                *data_changed = true;
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.link(RichText::new("Delete").font(Theme::font_sans(12.0)).color(Theme::FAINT)).clicked() {
                                    state.sub_to_delete = Some(sub.id);
                                }
                            });
                        });
                    });

                    ui.add_space(Theme::CARD_GAP);
                }

                // --- DASHED "+ Add subscription" FULL-WIDTH CARD AT END OF STACK ---
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
                    "+ Add subscription".to_string(),
                    Theme::font_sans(14.0),
                    Theme::ACCENT,
                );
                let text_pos = add_rect.center() - add_galley.size() / 2.0;
                ui.painter().galley(text_pos, add_galley, Theme::ACCENT);

                if add_resp.clicked() {
                    state.open_add(def_card);
                }

                if let Some(sub) = sub_to_edit {
                    state.open_edit(&sub);
                }

                ui.add_space(Theme::PAGE_MARGIN);
            });
        });
    });

    // --- DELETE CONFIRMATION MODAL ---
    if let Some(del_id) = state.sub_to_delete {
        egui::Window::new("Confirm Delete Subscription")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(300.0);
                ui.label("Are you sure you want to delete this subscription?");
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Delete").color(Theme::EXPENSE).strong()).clicked() {
                        data.subscriptions.retain(|s| s.id != del_id);
                        *data_changed = true;
                        state.sub_to_delete = None;
                    }
                    if ui.button("Cancel").clicked() {
                        state.sub_to_delete = None;
                    }
                });
            });
    }

    // --- ADD / EDIT SUBSCRIPTION MODAL ---
    if state.show_modal {
        let is_edit = state.editing_sub_id.is_some();
        let title = if is_edit { "Edit Subscription" } else { "Add Subscription" };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(360.0);
                ui.add_space(6.0);

                ui.label("Subscription Name:");
                ui.text_edit_singleline(&mut state.name);

                ui.add_space(6.0);
                ui.label("Amount (PKR):");
                ui.text_edit_singleline(&mut state.amount);

                ui.add_space(6.0);
                ui.label("Billing Cycle:");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut state.cycle_type, 0, "Monthly");
                    ui.selectable_value(&mut state.cycle_type, 1, "Yearly");
                    ui.selectable_value(&mut state.cycle_type, 2, "Custom Days");
                });

                if state.cycle_type == 2 {
                    ui.horizontal(|ui| {
                        ui.label("Every N days:");
                        ui.text_edit_singleline(&mut state.custom_days);
                    });
                }

                ui.add_space(6.0);
                ui.label("Start Date (YYYY-MM-DD):");
                ui.text_edit_singleline(&mut state.start_date);

                ui.add_space(6.0);
                ui.label("Linked Card / Account:");
                let current_card_name = data.cards
                    .iter()
                    .find(|c| Some(c.id) == state.linked_card_id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("Select Card");

                egui::ComboBox::from_id_salt("sub_card_dropdown")
                    .selected_text(current_card_name)
                    .show_ui(ui, |ui| {
                        for card in &data.cards {
                            if ui.selectable_label(state.linked_card_id == Some(card.id), &card.name).clicked() {
                                state.linked_card_id = Some(card.id);
                            }
                        }
                    });

                if let Some(err) = &state.modal_error {
                    ui.colored_label(Theme::EXPENSE, err);
                }

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Save Subscription").strong().color(Theme::ACCENT)).clicked() {
                        let name = state.name.trim().to_string();
                        if name.is_empty() {
                            state.modal_error = Some("Name cannot be empty.".to_string());
                            return;
                        }

                        let amount = match Theme::parse_pkr_input(&state.amount) {
                            Some(a) if a > 0.0 => a,
                            _ => {
                                state.modal_error = Some("Amount must be a positive number.".to_string());
                                return;
                            }
                        };

                        let start_date = match NaiveDate::parse_from_str(state.start_date.trim(), "%Y-%m-%d") {
                            Ok(d) => d,
                            Err(_) => {
                                state.modal_error = Some("Invalid start date. Use YYYY-MM-DD.".to_string());
                                return;
                            }
                        };

                        let cycle = match state.cycle_type {
                            0 => BillingCycle::Monthly,
                            1 => BillingCycle::Yearly,
                            _ => {
                                let days = state.custom_days.trim().parse::<u32>().unwrap_or(30).max(1);
                                BillingCycle::CustomDays(days)
                            }
                        };

                        let card_id = match state.linked_card_id {
                            Some(id) => id,
                            None => match data.cards.first() {
                                Some(c) => c.id,
                                None => {
                                    state.modal_error = Some("No cards available to link.".to_string());
                                    return;
                                }
                            },
                        };

                        let next_due = SubscriptionManager::advance_date(start_date, &cycle);

                        if let Some(sub_id) = state.editing_sub_id {
                            if let Some(sub) = data.subscriptions.iter_mut().find(|s| s.id == sub_id) {
                                sub.name = name;
                                sub.amount = amount;
                                sub.cycle = cycle;
                                sub.start_date = start_date;
                                sub.next_due_date = next_due;
                                sub.card_id = card_id;
                            }
                        } else {
                            let new_sub = Subscription {
                                id: Uuid::new_v4(),
                                card_id,
                                name,
                                amount,
                                cycle,
                                start_date,
                                next_due_date: next_due,
                                paused: false,
                            };
                            data.subscriptions.push(new_sub);
                        }

                        *data_changed = true;
                        state.show_modal = false;
                    }

                    if ui.button("Cancel").clicked() {
                        state.show_modal = false;
                    }
                });
            });
    }
}
