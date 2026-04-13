from PIL import Image, ImageDraw

# Create a 512x512 image with transparent background
img = Image.new("RGBA", (512, 512), (255, 255, 255, 0))
draw = ImageDraw.Draw(img)

# Colors
black = (0, 0, 0, 255)
white = (255, 255, 255, 255)

# Draw fish body (approximate ellipse)
# [x0, y0, x1, y1]
draw.ellipse([100, 150, 420, 380], fill=black)

# Draw tail (triangle)
# Points: (100, 265), (20, 180), (20, 350)
draw.polygon([(100, 265), (20, 180), (20, 350)], fill=black)

# Draw top fin
draw.polygon([(250, 180), (320, 120), (380, 180)], fill=black)

# Draw bottom fin
draw.polygon([(250, 350), (300, 420), (350, 350)], fill=black)

# Draw eye
draw.ellipse([330, 220, 370, 260], fill=white)

# Save as PNG
output_path = r"C:\Users\Lukinha Gaming\Documents\ia\sensus-tauri\src-tauri\icons\icon.png"
img.save(output_path, "PNG")
print(f"Fish icon created at {output_path}")
