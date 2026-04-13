from PIL import Image
import os

input_path = r"C:\Users\Lukinha Gaming\Documents\ia\sensus-tauri\src-tauri\icons\icon.png"
output_path = r"C:\Users\Lukinha Gaming\Documents\ia\sensus-tauri\src-tauri\icons\icon.ico"

try:
    img = Image.open(input_path)
    # ICO files usually contain multiple sizes. 
    # We'll provide a few common ones.
    icon_sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    img.save(output_path, format='ICO', sizes=icon_sizes)
    print(f"Successfully converted {input_path} to {output_path}")
except Exception as e:
    print(f"Error during conversion: {e}")
