use chrono::NaiveDate;
use egui::{Color32, FontId, RichText, Stroke, Ui};

#[allow(dead_code)]
pub struct Theme;

#[allow(dead_code)]
impl Theme {
    // --- EXACT CSS VARIABLES FROM APPROVED REFERENCE ---
    // --bg:#11141A; --panel:#171B23; --panel2:#1C212B; --border:#242938;
    pub const BG: Color32 = Color32::from_rgb(0x11, 0x14, 0x1A);
    pub const PANEL: Color32 = Color32::from_rgb(0x17, 0x1B, 0x23);
    pub const PANEL2: Color32 = Color32::from_rgb(0x1C, 0x21, 0x2B);
    pub const BORDER: Color32 = Color32::from_rgb(0x24, 0x29, 0x38);

    // --text:#E8EAF0; --muted:#7C8494; --faint:#4B5164;
    pub const TEXT: Color32 = Color32::from_rgb(0xE8, 0xEA, 0xF0);
    pub const MUTED: Color32 = Color32::from_rgb(0x7C, 0x84, 0x94);
    pub const FAINT: Color32 = Color32::from_rgb(0x4B, 0x51, 0x64);

    // --accent:#6C7CF0; --accent-dim:#3A3F73;
    pub const ACCENT: Color32 = Color32::from_rgb(0x6C, 0x7C, 0xF0);
    pub const ACCENT_DIM: Color32 = Color32::from_rgb(0x3A, 0x3F, 0x73);

    // --in:#3FCDA8; --out:#F2726B;
    pub const IN: Color32 = Color32::from_rgb(0x3F, 0xCD, 0xA8);
    pub const OUT: Color32 = Color32::from_rgb(0xF2, 0x72, 0x6B);

    // Star & Badge gold (#E8B93F)
    pub const STAR: Color32 = Color32::from_rgb(0xE8, 0xB9, 0x3F);
    // Correct premultiplied values for 10% opacity: 232*0.10=23, 185*0.10=18, 63*0.10=6, alpha=26
    pub const BADGE_BG: Color32 = Color32::from_rgba_premultiplied(23, 18, 6, 26);

    // Tinted button backgrounds (accurate premultiplied alpha at 12% opacity)
    // 63*0.12=7, 205*0.12=25, 168*0.12=20, alpha=31
    pub const IN_BTN_BG: Color32 = Color32::from_rgba_premultiplied(7, 25, 20, 31);
    // 242*0.12=29, 114*0.12=14, 107*0.12=13, alpha=31
    pub const OUT_BTN_BG: Color32 = Color32::from_rgba_premultiplied(29, 14, 13, 31);

    // Aliases matching earlier code and clear semantics
    pub const BG_PANEL: Color32 = Self::PANEL;
    pub const BG_PANEL2: Color32 = Self::PANEL2;
    pub const BG_CARD: Color32 = Self::PANEL;
    pub const BG_HOVER: Color32 = Self::PANEL2;
    pub const TEXT_PRIMARY: Color32 = Self::TEXT;
    pub const TEXT_MUTED: Color32 = Self::MUTED;
    pub const TEXT_DIM: Color32 = Self::FAINT;
    pub const TEXT_FAINT: Color32 = Self::FAINT;
    pub const ACCENT_BG: Color32 = Self::ACCENT_DIM;
    pub const INCOME: Color32 = Self::IN;
    pub const EXPENSE: Color32 = Self::OUT;
    pub const INCOME_BG: Color32 = Self::IN_BTN_BG;
    pub const EXPENSE_BG: Color32 = Self::OUT_BTN_BG;
    pub const INCOME_BORDER: Color32 = Self::BORDER;
    pub const EXPENSE_BORDER: Color32 = Self::BORDER;

    // --- EXACT RADII ---
    pub const RADIUS_PANEL: f32 = 12.0;
    pub const RADIUS_CARD: f32 = 12.0;
    pub const RADIUS_TAB: f32 = 8.0;
    pub const RADIUS_BTN: f32 = 7.0;
    pub const RADIUS_SM: f32 = 7.0;
    pub const RADIUS_MD: f32 = 12.0;
    pub const RADIUS_PILL: f32 = 8.0;
    pub const RADIUS_BADGE: f32 = 6.0;
    pub const RADIUS_BAR: f32 = 4.0;

    // --- SPACING CONSTANTS ---
    pub const CARD_GAP: f32 = 16.0;
    pub const CARD_PADDING: f32 = 18.0;
    pub const CONTENT_MAX_WIDTH: f32 = 900.0;
    pub const PAGE_MARGIN: f32 = 24.0;

    pub fn stroke_border() -> Stroke {
        Stroke::new(1.0_f32, Self::BORDER)
    }

    pub fn panel_stroke() -> Stroke {
        Stroke::new(1.0_f32, Self::BORDER)
    }

    pub fn stroke_accent_dim() -> Stroke {
        Stroke::new(1.0_f32, Self::ACCENT_DIM)
    }

    pub fn stroke_dashed() -> Stroke {
        Stroke::new(1.0_f32, Self::BORDER)
    }

    pub fn dashed_stroke() -> Stroke {
        Stroke::new(1.0_f32, Self::ACCENT)
    }

    pub fn money_label(ui: &mut Ui, text: &str, color: Color32, size: f32) -> egui::Response {
        Self::render_mono(ui, text, color, size)
    }

    // --- EXACT TYPOGRAPHY ---
    // --head: 'Space Grotesk'
    pub fn font_head(size: f32) -> FontId {
        FontId::new(size, egui::FontFamily::Name("SpaceGrotesk".into()))
    }

    // --sans: 'Inter'
    pub fn font_sans(size: f32) -> FontId {
        FontId::new(size, egui::FontFamily::Proportional)
    }

    // --mono: 'IBM Plex Mono'
    pub fn font_mono(size: f32) -> FontId {
        FontId::new(size, egui::FontFamily::Name("IBMPlexMono".into()))
    }

    /// Formats a float as currency in Pakistani Rupees (PKR) with thousands separators.
    /// Example: 1234567.89 -> "Rs 1,234,567.89"
    pub fn format_pkr(amount: f64) -> String {
        let is_neg = amount < 0.0;
        let abs_val = amount.abs();
        let int_part = abs_val.trunc() as i64;
        let frac_part = ((abs_val.fract() * 100.0).round() as i64).min(99);

        let int_str = int_part.to_string();
        let mut formatted_int = String::new();
        let chars: Vec<char> = int_str.chars().collect();
        let len = chars.len();

        for (idx, &ch) in chars.iter().enumerate() {
            if idx > 0 && (len - idx) % 3 == 0 {
                formatted_int.push(',');
            }
            formatted_int.push(ch);
        }

        if is_neg {
            format!("-Rs {}.{:02}", formatted_int, frac_part)
        } else {
            format!("Rs {}.{:02}", formatted_int, frac_part)
        }
    }

    /// Formats whole PKR amount without decimals: "Rs 106,736"
    pub fn format_pkr_whole(amount: f64) -> String {
        let is_neg = amount < 0.0;
        let abs_val = amount.abs().round() as i64;
        let int_str = abs_val.to_string();
        let mut formatted_int = String::new();
        let chars: Vec<char> = int_str.chars().collect();
        let len = chars.len();

        for (idx, &ch) in chars.iter().enumerate() {
            if idx > 0 && (len - idx) % 3 == 0 {
                formatted_int.push(',');
            }
            formatted_int.push(ch);
        }

        if is_neg {
            format!("-Rs {}", formatted_int)
        } else {
            format!("Rs {}", formatted_int)
        }
    }

    /// Parses user-entered PKR currency text by stripping commas and whitespace.
    /// Returns None if input is invalid (letters, multiple decimal points, etc.).
    pub fn parse_pkr_input(raw: &str) -> Option<f64> {
        let cleaned: String = raw.chars().filter(|&c| c != ',').collect();
        cleaned.trim().parse::<f64>().ok()
    }

    pub fn format_date(date: &NaiveDate) -> String {
        date.format("%d %b %Y").to_string()
    }

    /// Renders monetary figure in IBM Plex Mono font.
    pub fn render_mono(ui: &mut Ui, text: &str, color: Color32, size: f32) -> egui::Response {
        ui.label(RichText::new(text).font(Self::font_mono(size)).color(color))
    }
}
