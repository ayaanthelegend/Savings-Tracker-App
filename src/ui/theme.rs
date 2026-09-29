use chrono::{Datelike, NaiveDate};
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

    // NEW: categorical palette for spending categories and chart series
    pub const CAT_INCOME: Color32 = Color32::from_rgb(0x6C, 0x7C, 0xF0);
    pub const CAT_FOOD: Color32 = Color32::from_rgb(0x3F, 0xCD, 0xA8);
    pub const CAT_SUBSCRIPTION: Color32 = Color32::from_rgb(0xF2, 0xA6, 0x3F);
    pub const CAT_SHOPPING: Color32 = Color32::from_rgb(0xE3, 0x6B, 0xB3);
    pub const CAT_TRANSPORT: Color32 = Color32::from_rgb(0x5F, 0xB8, 0xE0);
    pub const CAT_OTHER: Color32 = Color32::from_rgb(0x9C, 0x8C, 0xF0);

    // NEW: subtle grid texture line (rgba(255,255,255,0.04))
    pub const GRID_LINE: Color32 = Color32::from_rgba_premultiplied(10, 10, 10, 10);

    // Savings Plan status badge colors
    pub const STATUS_ON_TRACK: Color32 = Color32::from_rgb(0x3F, 0xCD, 0xA8);
    pub const STATUS_BEHIND: Color32 = Color32::from_rgb(0xF2, 0xA6, 0x3F);
    pub const STATUS_AHEAD: Color32 = Color32::from_rgb(0x52, 0xF2, 0xC5);

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

    /// Formats date for display: "4 Sep 2026"
    pub fn format_date(date: &NaiveDate) -> String {
        format!("{} {}", date.day(), date.format("%b %Y"))
    }

    /// Formats date for user input fields as M/D/YYYY (e.g. "9/4/2026")
    pub fn format_input_date(date: &NaiveDate) -> String {
        format!("{}/{}/{}", date.month(), date.day(), date.year())
    }

    /// Parses user-entered date string supporting M/D/YYYY, D/M/YYYY, YYYY-MM-DD, and D Mon YYYY.
    pub fn parse_date_input(raw: &str) -> Option<NaiveDate> {
        let s = raw.trim();
        if s.is_empty() {
            return None;
        }

        // Try standard chrono formats first
        if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Some(d);
        }
        if let Ok(d) = NaiveDate::parse_from_str(s, "%d %b %Y") {
            return Some(d);
        }
        if let Ok(d) = NaiveDate::parse_from_str(s, "%e %b %Y") {
            return Some(d);
        }

        // Split by delimiter '/', '-', or '.'
        let parts: Vec<&str> = s.split(|c| c == '/' || c == '-' || c == '.').collect();
        if parts.len() == 3 {
            // Case 1: M/D/YYYY (or D/M/YYYY fallback)
            if let (Ok(p1), Ok(p2), Ok(p3)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<i32>(),
            ) {
                if p3 >= 1900 && p3 <= 2200 {
                    // Try Month = p1, Day = p2 first (per user requirement: 9/4/2026 is month 9, day 4)
                    if let Some(d) = NaiveDate::from_ymd_opt(p3, p1, p2) {
                        return Some(d);
                    }
                    // Fallback to Day = p1, Month = p2 if p1 > 12
                    if let Some(d) = NaiveDate::from_ymd_opt(p3, p2, p1) {
                        return Some(d);
                    }
                }
            }

            // Case 2: YYYY/M/D
            if let (Ok(y), Ok(m), Ok(d)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                if y >= 1900 && y <= 2200 {
                    if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
                        return Some(date);
                    }
                }
            }
        }

        None
    }

    // --hero: 'IBM Plex Mono' 40-48px
    pub fn font_hero(size: f32) -> FontId {
        FontId::new(size, egui::FontFamily::Name("IBMPlexMono".into()))
    }

    /// Fixed categorical palette lookup for spending categories and chart series
    pub fn category_color(name: &str) -> Color32 {
        category_color(name)
    }

    /// Paints a very subtle grid pattern background (24px interval) using GRID_LINE
    pub fn paint_grid_background(ui: &egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        let step = 24.0;
        let stroke = Stroke::new(1.0_f32, Self::GRID_LINE);

        let start_x = (rect.left() / step).floor() * step;
        let mut x = start_x;
        while x <= rect.right() {
            if x >= rect.left() {
                painter.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], stroke);
            }
            x += step;
        }

        let start_y = (rect.top() / step).floor() * step;
        let mut y = start_y;
        while y <= rect.bottom() {
            if y >= rect.top() {
                painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], stroke);
            }
            y += step;
        }
    }

    /// Paints a 3px colored bar along the top edge of a card frame
    pub fn paint_card_accent_bar(painter: &egui::Painter, card_rect: egui::Rect, color: Color32, radius: f32) {
        let bar_rect = egui::Rect::from_min_size(card_rect.left_top(), egui::vec2(card_rect.width(), 3.0));
        painter.rect_filled(
            bar_rect,
            egui::Rounding {
                nw: radius,
                ne: radius,
                sw: 0.0,
                se: 0.0,
            },
            color,
        );
    }

    /// Renders monetary figure in IBM Plex Mono font.
    pub fn render_mono(ui: &mut Ui, text: &str, color: Color32, size: f32) -> egui::Response {
        ui.label(RichText::new(text).font(Self::font_mono(size)).color(color))
    }
}

/// Module-level helper for category color lookup (Section 1.1)
pub fn category_color(name: &str) -> Color32 {
    match name.to_lowercase().trim() {
        "income" => Theme::CAT_INCOME,
        "food" => Theme::CAT_FOOD,
        "subscription" | "subscriptions" => Theme::CAT_SUBSCRIPTION,
        "shopping" => Theme::CAT_SHOPPING,
        "transport" | "transportation" => Theme::CAT_TRANSPORT,
        "other" => Theme::CAT_OTHER,
        _ => {
            let palette = [
                Theme::CAT_INCOME,
                Theme::CAT_FOOD,
                Theme::CAT_SUBSCRIPTION,
                Theme::CAT_SHOPPING,
                Theme::CAT_TRANSPORT,
                Theme::CAT_OTHER,
            ];
            let hash = name.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize));
            palette[hash % palette.len()]
        }
    }
}
