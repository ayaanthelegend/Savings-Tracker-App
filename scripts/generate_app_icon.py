import math
import os
from PIL import Image, ImageDraw, ImageFont, ImageFilter

def create_app_icon():
    CANVAS = 1024
    scale = CANVAS / 256.0 # 4.0
    
    # Create RGBA master canvas
    im = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    
    # 1. Base Squircle Geometry
    # Padding 36px on 1024 -> 952x952 squircle
    pad = int(40 * scale / 4.0)
    size = CANVAS - 2 * pad
    radius = int(220 * scale / 4.0)
    
    # Smooth drop shadow under squircle
    shadow_img = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    s_draw = ImageDraw.Draw(shadow_img)
    s_offset = int(20 * scale / 4.0)
    s_draw.rounded_rectangle(
        [pad + 6, pad + s_offset, pad + size - 6, pad + size + s_offset],
        radius=radius,
        fill=(0, 0, 0, 160)
    )
    shadow_img = shadow_img.filter(ImageFilter.GaussianBlur(radius=int(26 * scale / 4.0)))
    im.alpha_composite(shadow_img)
    
    # Squircle mask
    base_mask = Image.new("L", (CANVAS, CANVAS), 0)
    m_draw = ImageDraw.Draw(base_mask)
    m_draw.rounded_rectangle(
        [pad, pad, pad + size, pad + size],
        radius=radius,
        fill=255
    )
    
    # Background gradient: #19202E (top) -> #0D1017 (bottom)
    grad = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    g_pix = grad.load()
    top_c = (25, 32, 46)
    bot_c = (13, 16, 23)
    for y in range(CANVAS):
        t = y / CANVAS
        r = int(top_c[0] + (bot_c[0] - top_c[0]) * t)
        g = int(top_c[1] + (bot_c[1] - top_c[1]) * t)
        b = int(top_c[2] + (bot_c[2] - top_c[2]) * t)
        for x in range(CANVAS):
            g_pix[x, y] = (r, g, b, 255)
            
    squircle_layer = Image.composite(grad, Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0)), base_mask)
    
    # Subtle signature 24px ledger grid texture inside squircle
    grid_layer = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    grid_draw = ImageDraw.Draw(grid_layer)
    grid_step = int(60 * scale / 4.0)
    for x in range(pad, pad + size, grid_step):
        grid_draw.line([(x, pad), (x, pad + size)], fill=(255, 255, 255, 14), width=2)
    for y in range(pad, pad + size, grid_step):
        grid_draw.line([(pad, y), (pad + size, y)], fill=(255, 255, 255, 14), width=2)
    squircle_layer.alpha_composite(Image.composite(grid_layer, Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0)), base_mask))
    
    # Clean subtle glowing squircle border
    border_img = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    b_draw = ImageDraw.Draw(border_img)
    b_draw.rounded_rectangle(
        [pad, pad, pad + size, pad + size],
        radius=radius,
        outline=(108, 124, 240, 110), # #6C7CF0 accent outline
        width=int(5 * scale / 4.0)
    )
    # Subtle top edge highlight inside squircle
    b_draw.rounded_rectangle(
        [pad + 3, pad + 3, pad + size - 3, pad + 12],
        radius=radius,
        fill=(255, 255, 255, 25)
    )
    squircle_layer.alpha_composite(Image.composite(border_img, Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0)), base_mask))
    
    im.alpha_composite(squircle_layer)
    
    # 2. Main Visual Subject:
    # Sleek modern Banking Card (angled slightly or centered)
    # Ascending Growth Signal Bars
    # Radiant Emerald & Gold Coin with bold "Rs"
    
    # Card layout:
    card_x0 = int(180 * scale / 4.0)
    card_y0 = int(410 * scale / 4.0)
    card_w = int(664 * scale / 4.0)
    card_h = int(400 * scale / 4.0)
    card_r = int(54 * scale / 4.0)
    
    # Card drop shadow
    card_shadow = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    cs_draw = ImageDraw.Draw(card_shadow)
    cs_draw.rounded_rectangle(
        [card_x0, card_y0 + 18, card_x0 + card_w, card_y0 + card_h + 18],
        radius=card_r,
        fill=(0, 0, 0, 180)
    )
    card_shadow = card_shadow.filter(ImageFilter.GaussianBlur(radius=int(26 * scale / 4.0)))
    im.alpha_composite(card_shadow)
    
    # Card surface
    card_layer = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    cd_draw = ImageDraw.Draw(card_layer)
    
    # Card body fill: deep slate #1C2333
    cd_draw.rounded_rectangle(
        [card_x0, card_y0, card_x0 + card_w, card_y0 + card_h],
        radius=card_r,
        fill=(28, 35, 51, 255),
        outline=(58, 70, 99, 255),
        width=int(6 * scale / 4.0)
    )
    
    # Top 3px accent bar (Redesign signature feature #6C7CF0)
    top_bar_h = int(28 * scale / 4.0)
    cd_draw.rounded_rectangle(
        [card_x0, card_y0, card_x0 + card_w, card_y0 + top_bar_h + 16],
        radius=card_r,
        fill=(108, 124, 240, 255)
    )
    # Clip lower portion of the rounded rect
    cd_draw.rectangle(
        [card_x0, card_y0 + top_bar_h, card_x0 + card_w, card_y0 + card_h - card_r],
        fill=(28, 35, 51, 255)
    )
    # Re-stroke border
    cd_draw.rounded_rectangle(
        [card_x0, card_y0, card_x0 + card_w, card_y0 + card_h],
        radius=card_r,
        outline=(58, 70, 99, 255),
        width=int(6 * scale / 4.0)
    )
    
    # Gold EMV Chip
    chip_x = card_x0 + int(70 * scale / 4.0)
    chip_y = card_y0 + int(105 * scale / 4.0)
    chip_w = int(116 * scale / 4.0)
    chip_h = int(90 * scale / 4.0)
    chip_r = int(20 * scale / 4.0)
    cd_draw.rounded_rectangle(
        [chip_x, chip_y, chip_x + chip_w, chip_y + chip_h],
        radius=chip_r,
        fill=(242, 166, 63, 255), # #F2A63F
        outline=(255, 218, 120, 255),
        width=int(5 * scale / 4.0)
    )
    # Chip circuit detail
    cd_draw.line([(chip_x + int(chip_w * 0.4), chip_y), (chip_x + int(chip_w * 0.4), chip_y + chip_h)], fill=(190, 125, 30, 255), width=3)
    cd_draw.line([(chip_x, chip_y + int(chip_h * 0.5)), (chip_x + chip_w, chip_y + int(chip_h * 0.5))], fill=(190, 125, 30, 255), width=3)
    
    # Card numbers / dots simulation
    dot_y = card_y0 + int(250 * scale / 4.0)
    for g in range(3):
        group_x = card_x0 + int(70 * scale / 4.0) + g * int(115 * scale / 4.0)
        for d in range(4):
            dx = group_x + d * int(18 * scale / 4.0)
            cd_draw.ellipse([dx, dot_y, dx + int(9 * scale / 4.0), dot_y + int(9 * scale / 4.0)], fill=(124, 132, 148, 170))
            
    # Ascending Financial Growth Bars
    bars_base_y = card_y0 + int(346 * scale / 4.0)
    bar_w = int(38 * scale / 4.0)
    bar_gap = int(22 * scale / 4.0)
    bar_start_x = card_x0 + int(430 * scale / 4.0)
    bar_heights = [int(h * scale / 4.0) for h in [75, 130, 195, 260]]
    bar_colors = [
        (95, 184, 224, 230),  # Transport sky blue
        (108, 124, 240, 245), # Indigo accent
        (63, 205, 168, 255),  # Emerald teal
        (63, 205, 168, 255)   # Emerald teal
    ]
    
    for i, (h, col) in enumerate(zip(bar_heights, bar_colors)):
        bx = bar_start_x + i * (bar_w + bar_gap)
        by = bars_base_y - h
        cd_draw.rounded_rectangle(
            [bx, by, bx + bar_w, bars_base_y],
            radius=int(12 * scale / 4.0),
            fill=col
        )
        
    # Dynamic trend arrow connecting tops of bars
    trend_pts = [
        (bar_start_x + int(19 * scale / 4.0), bars_base_y - bar_heights[0] - int(16 * scale / 4.0)),
        (bar_start_x + (bar_w + bar_gap) + int(19 * scale / 4.0), bars_base_y - bar_heights[1] - int(16 * scale / 4.0)),
        (bar_start_x + 2 * (bar_w + bar_gap) + int(19 * scale / 4.0), bars_base_y - bar_heights[2] - int(16 * scale / 4.0)),
        (bar_start_x + 3 * (bar_w + bar_gap) + int(19 * scale / 4.0), bars_base_y - bar_heights[3] - int(20 * scale / 4.0)),
    ]
    for i in range(len(trend_pts) - 1):
        cd_draw.line([trend_pts[i], trend_pts[i+1]], fill=(63, 205, 168, 255), width=int(9 * scale / 4.0))
        
    # Glowing trend peak dot
    peak = trend_pts[-1]
    peak_r = int(14 * scale / 4.0)
    cd_draw.ellipse([peak[0] - peak_r, peak[1] - peak_r, peak[0] + peak_r, peak[1] + peak_r], fill=(255, 255, 255, 255))
    
    im.alpha_composite(card_layer)
    
    # 3. Radiant Savings Coin with "Rs"
    # Positioned at upper-center right
    coin_cx = int(680 * scale / 4.0)
    coin_cy = int(320 * scale / 4.0)
    coin_r = int(172 * scale / 4.0)
    
    # Coin ambient outer glow
    coin_glow = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    cg_draw = ImageDraw.Draw(coin_glow)
    cg_draw.ellipse(
        [coin_cx - coin_r - 32, coin_cy - coin_r - 32, coin_cx + coin_r + 32, coin_cy + coin_r + 32],
        fill=(63, 205, 168, 140)
    )
    coin_glow = coin_glow.filter(ImageFilter.GaussianBlur(radius=int(32 * scale / 4.0)))
    im.alpha_composite(coin_glow)
    
    # Coin drop shadow
    coin_sh = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    csh_draw = ImageDraw.Draw(coin_sh)
    csh_draw.ellipse(
        [coin_cx - coin_r, coin_cy - coin_r + 20, coin_cx + coin_r, coin_cy + coin_r + 20],
        fill=(0, 0, 0, 200)
    )
    coin_sh = coin_sh.filter(ImageFilter.GaussianBlur(radius=int(22 * scale / 4.0)))
    im.alpha_composite(coin_sh)
    
    # Coin body
    coin_layer = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    cn_draw = ImageDraw.Draw(coin_layer)
    
    # Outer coin rim: Emerald #3FCDA8 with highlight
    cn_draw.ellipse(
        [coin_cx - coin_r, coin_cy - coin_r, coin_cx + coin_r, coin_cy + coin_r],
        fill=(63, 205, 168, 255),
        outline=(125, 245, 215, 255),
        width=int(8 * scale / 4.0)
    )
    
    # Inner coin face: Deep midnight emerald #112922
    inner_r = int(140 * scale / 4.0)
    cn_draw.ellipse(
        [coin_cx - inner_r, coin_cy - inner_r, coin_cx + inner_r, coin_cy + inner_r],
        fill=(17, 41, 34, 255),
        outline=(85, 230, 190, 200),
        width=int(5 * scale / 4.0)
    )
    
    # Render bold "Rs" in center of coin
    font_size = int(148 * scale / 4.0)
    font = ImageFont.truetype("assets/fonts/Inter.ttf", font_size)
    symbol = "Rs"
    bbox = font.getbbox(symbol)
    text_w = bbox[2] - bbox[0]
    text_h = bbox[3] - bbox[1]
    
    tx = coin_cx - text_w / 2 - bbox[0]
    ty = coin_cy - text_h / 2 - bbox[1] - int(4 * scale / 4.0)
    
    # Shadow behind text
    cn_draw.text((tx + 3, ty + 6), symbol, font=font, fill=(0, 0, 0, 160), stroke_width=int(7 * scale / 4.0), stroke_fill=(0, 0, 0, 160))
    # Crisp white/mint bold text
    cn_draw.text((tx, ty), symbol, font=font, fill=(235, 253, 247, 255), stroke_width=int(7 * scale / 4.0), stroke_fill=(235, 253, 247, 255))
    
    # Sparkle star on upper-left coin rim
    star_cx = coin_cx - int(coin_r * 0.70)
    star_cy = coin_cy - int(coin_r * 0.70)
    star_r = int(22 * scale / 4.0)
    cn_draw.ellipse([star_cx - star_r, star_cy - star_r, star_cx + star_r, star_cy + star_r], fill=(255, 255, 255, 240))
    # Star rays
    cn_draw.line([(star_cx - int(34 * scale / 4.0), star_cy), (star_cx + int(34 * scale / 4.0), star_cy)], fill=(255, 255, 255, 200), width=int(3 * scale / 4.0))
    cn_draw.line([(star_cx, star_cy - int(34 * scale / 4.0)), (star_cx, star_cy + int(34 * scale / 4.0))], fill=(255, 255, 255, 200), width=int(3 * scale / 4.0))
    
    im.alpha_composite(coin_layer)
    
    # Save master high-resolution 1024x1024
    im.save("assets/icon-1024.png", "PNG")
    
    # Generate 512, 256, 128, 64, 48, 32, 24, 16 sizes
    sizes = [512, 256, 128, 64, 48, 32, 24, 16]
    resized_images = {}
    for sz in sizes:
        resized = im.resize((sz, sz), Image.Resampling.LANCZOS)
        resized.save(f"assets/icon-{sz}.png", "PNG")
        resized_images[sz] = resized
        
    # Primary assets/icon.png used for eframe runtime loading
    resized_images[256].save("assets/icon.png", "PNG")
    
    # Windows Multi-resolution ICO file
    ico_sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    im.save(
        "assets/icon.ico",
        format="ICO",
        sizes=ico_sizes
    )
    print("Regenerated clean, professional app icon assets!")

if __name__ == "__main__":
    create_app_icon()
