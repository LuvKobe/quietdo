"""生成静待 QuietDo 应用图标：透明圆角 + 深色渐变底 + 居中蓝色对勾。
输出 design/app-icon.png（1024x1024，带透明通道），供 `tauri icon` 生成全套尺寸。
"""
from PIL import Image, ImageDraw

SIZE = 1024
# 圆角矩形铺满画布，仅留极小 padding，圆角之外透明（消除桌面黑边）
PAD = 24
RADIUS = 230           # macOS squircle 观感的圆角半径
ACCENT = (10, 132, 255)  # Apple 系统蓝 #0A84FF

# ---- 1. 透明画布 ----
img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))

# ---- 2. 深色竖直渐变作为图标底色 ----
grad = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
top = (44, 44, 48)     # #2c2c30
bottom = (22, 22, 24)  # #161618
for y in range(SIZE):
    t = y / (SIZE - 1)
    r = int(top[0] + (bottom[0] - top[0]) * t)
    g = int(top[1] + (bottom[1] - top[1]) * t)
    b = int(top[2] + (bottom[2] - top[2]) * t)
    for_line = (r, g, b, 255)
    grad.paste(for_line, (0, y, SIZE, y + 1))

# ---- 3. 圆角矩形蒙版，把渐变裁成圆角，角落保持透明 ----
mask = Image.new("L", (SIZE, SIZE), 0)
mdraw = ImageDraw.Draw(mask)
mdraw.rounded_rectangle([PAD, PAD, SIZE - PAD, SIZE - PAD], radius=RADIUS, fill=255)
img.paste(grad, (0, 0), mask)

# ---- 4. 居中蓝色对勾（粗线条，圆头）----
draw = ImageDraw.Draw(img)
# 对勾三点（相对画布），偏移让视觉居中
p1 = (SIZE * 0.33, SIZE * 0.50)
p2 = (SIZE * 0.45, SIZE * 0.62)
p3 = (SIZE * 0.69, SIZE * 0.37)
lw = 78
draw.line([p1, p2, p3], fill=ACCENT, width=lw, joint="curve")
# 线端画圆点，形成圆头效果
for (cx, cy) in (p1, p3):
    draw.ellipse([cx - lw / 2, cy - lw / 2, cx + lw / 2, cy + lw / 2], fill=ACCENT)

img.save("design/app-icon.png")
print("saved design/app-icon.png", img.size)
