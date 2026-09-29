use crate::auth::{AuthManager, AuthSession, SupabaseConfig};
use crate::ui::theme::Theme;
use eframe::egui;
use egui::{Color32, RichText, Rounding, Stroke, Ui, Vec2};
use std::time::Duration;

pub struct LoginViewState {
    pub email: String,
    pub password: String,
    pub supabase_url: String,
    pub supabase_key: String,
    pub is_signup: bool,
    pub is_submitting: bool,
    pub error_message: Option<String>,
    pub info_message: Option<String>,
}

impl Default for LoginViewState {
    fn default() -> Self {
        let config = AuthManager::load_config();
        Self {
            email: String::new(),
            password: String::new(),
            supabase_url: config.url,
            supabase_key: config.anon_key,
            is_signup: false,
            is_submitting: false,
            error_message: None,
            info_message: None,
        }
    }
}

pub enum LoginAction {
    LoggedIn(AuthSession),
    None,
}

#[allow(unused_assignments)]
pub fn render_login_view(ui: &mut Ui, state: &mut LoginViewState) -> LoginAction {
    let mut action = LoginAction::None;

    let avail_width = ui.available_width();
    let avail_height = ui.available_height();

    // Fixed card width ~420px, staying fixed and centered regardless of window size
    let card_width = 420.0f32.min(avail_width - 32.0).max(300.0);
    let h_margin = ((avail_width - card_width) / 2.0).max(16.0);
    let top_margin = ((avail_height - 420.0) / 2.0).clamp(24.0, 110.0);

    ui.add_space(top_margin);

    ui.horizontal(|ui| {
        ui.add_space(h_margin);

        ui.vertical(|ui| {
            ui.set_width(card_width);
            ui.set_max_width(card_width);

            egui::Frame::none()
                .fill(Theme::PANEL)
                .stroke(Stroke::new(1.0_f32, Theme::BORDER))
                .rounding(Rounding::same(Theme::RADIUS_CARD))
                .inner_margin(egui::Margin::symmetric(28.0, 26.0))
                .show(ui, |ui| {
                    let inner_width = ui.available_width();
                    ui.set_width(inner_width);
                    ui.set_max_width(inner_width);

                    // a. Brand row: dot + "Savings Tracker"
                    ui.horizontal_centered(|ui| {
                        let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), egui::Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 5.0, Theme::ACCENT);
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Savings Tracker")
                                .font(Theme::font_head(20.0))
                                .strong()
                                .color(Theme::TEXT),
                        );
                    });

                    // b. Subtitle
                    ui.add_space(6.0);
                    ui.horizontal_centered(|ui| {
                        ui.label(
                            RichText::new("Cloud Sync & Multi-Device Access")
                                .font(Theme::font_sans(12.5))
                                .color(Theme::MUTED),
                        );
                    });

                    // c. ~20px gap
                    ui.add_space(20.0);

                    // Inline Error Banner
                    if let Some(err) = &state.error_message {
                        egui::Frame::none()
                            .fill(Theme::OUT_BTN_BG)
                            .stroke(Stroke::new(1.0_f32, Theme::OUT))
                            .rounding(Rounding::same(Theme::RADIUS_BTN))
                            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("⚠").font(Theme::font_sans(12.5)).color(Theme::OUT));
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(err)
                                            .font(Theme::font_sans(12.0))
                                            .color(Theme::OUT),
                                    );
                                });
                            });
                        ui.add_space(14.0);
                    }

                    // Inline Info Banner
                    if let Some(info) = &state.info_message {
                        egui::Frame::none()
                            .fill(Theme::PANEL2)
                            .stroke(Stroke::new(1.0_f32, Theme::ACCENT))
                            .rounding(Rounding::same(Theme::RADIUS_BTN))
                            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(
                                    RichText::new(info)
                                        .font(Theme::font_sans(12.0))
                                        .color(Theme::ACCENT),
                                );
                            });
                        ui.add_space(14.0);
                    }

                    // Email field
                    ui.label(RichText::new("Email").font(Theme::font_sans(12.5)).color(Theme::MUTED));
                    ui.add_space(4.0);
                    let email_resp = ui.add_sized(
                        Vec2::new(inner_width, 32.0),
                        egui::TextEdit::singleline(&mut state.email)
                            .hint_text("user@example.com")
                            .font(Theme::font_sans(13.0)),
                    );

                    ui.add_space(12.0);

                    // Password field
                    ui.label(RichText::new("Password").font(Theme::font_sans(12.5)).color(Theme::MUTED));
                    ui.add_space(4.0);
                    let pass_resp = ui.add_sized(
                        Vec2::new(inner_width, 32.0),
                        egui::TextEdit::singleline(&mut state.password)
                            .password(true)
                            .hint_text("••••••••")
                            .font(Theme::font_sans(13.0)),
                    );

                    ui.add_space(18.0);

                    // Primary Button: Log In / Create Account
                    let btn_label = if state.is_submitting {
                        if state.is_signup { "Creating account..." } else { "Logging in..." }
                    } else if state.is_signup {
                        "Create Account"
                    } else {
                        "Log In"
                    };

                    let can_submit = !state.is_submitting;
                    let mut submit_clicked = false;

                    ui.add_enabled_ui(can_submit, |ui| {
                        let btn = egui::Button::new(
                            RichText::new(btn_label)
                                .font(Theme::font_sans(14.0))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Theme::ACCENT)
                        .rounding(Rounding::same(Theme::RADIUS_BTN));

                        if ui.add_sized(Vec2::new(inner_width, 38.0), btn).clicked() {
                            submit_clicked = true;
                        }
                    });

                    let enter_pressed = (email_resp.lost_focus() || pass_resp.lost_focus())
                        && ui.input(|i| i.key_pressed(egui::Key::Enter));

                    if (submit_clicked || enter_pressed) && can_submit {
                        state.error_message = None;
                        state.info_message = None;

                        let email = state.email.trim();
                        let pass = state.password.trim();

                        if email.is_empty() || pass.is_empty() {
                            state.error_message = Some("Please enter both email and password.".to_string());
                        } else {
                            let mut config = SupabaseConfig {
                                url: state.supabase_url.trim().to_string(),
                                anon_key: state.supabase_key.trim().to_string(),
                            };

                            // Automatically assume and use default project settings if not set
                            if config.url.is_empty() || config.anon_key.is_empty() {
                                config = AuthManager::load_config();
                            }

                            if config.url.is_empty() || config.anon_key.is_empty() {
                                state.error_message = Some("Backend project configuration is missing.".to_string());
                            } else {
                                state.is_submitting = true;
                                let client = reqwest::blocking::Client::builder()
                                    .timeout(Duration::from_secs(12))
                                    .build()
                                    .unwrap_or_default();

                                if state.is_signup {
                                    match AuthManager::signup(&client, &config, email, pass) {
                                        Ok(session) => {
                                            let _ = AuthManager::save_config(&config);
                                            action = LoginAction::LoggedIn(session);
                                        }
                                        Err(e) => {
                                            state.error_message = Some(e.to_string());
                                        }
                                    }
                                } else {
                                    match AuthManager::login(&client, &config, email, pass) {
                                        Ok(session) => {
                                            let _ = AuthManager::save_config(&config);
                                            action = LoginAction::LoggedIn(session);
                                        }
                                        Err(e) => {
                                            state.error_message = Some(e.to_string());
                                        }
                                    }
                                }
                                state.is_submitting = false;
                            }
                        }
                    }

                    // Secondary text link / toggle
                    ui.add_space(14.0);
                    ui.horizontal_centered(|ui| {
                        let toggle_prompt = if state.is_signup {
                            "Already have an account? Log in"
                        } else {
                            "Don't have an account? Sign up"
                        };
                        if ui.link(RichText::new(toggle_prompt).font(Theme::font_sans(12.5)).color(Theme::ACCENT)).clicked() {
                            state.is_signup = !state.is_signup;
                            state.error_message = None;
                            state.info_message = None;
                        }
                    });

                    ui.add_space(16.0);
                    ui.horizontal_centered(|ui| {
                        ui.label(
                            RichText::new("Data is stored locally and synchronized securely.")
                                .font(Theme::font_sans(11.0))
                                .color(Theme::FAINT),
                        );
                    });
                });
        });
    });

    action
}
