# Photo Tools



We can use imagemagick.

Examples:

```bash

magick identify -format "%f,%w,%h\n" ~/Pictures/100_FUJI/2025-12-21\ Nakamichi/* 

magick input.jpg -resize 800x800 output.jpg

# Shink-only (don't enlarge)
magick input.jpg -resize "800x800>" output.jpg


magick mogrify -path ../b/ -resize "800x800>" *


magick mogrify -path b/ -resize "800x800>" a/*
```

## Summary

You can resize images from the command line using several powerful tools, depending on your operating system. 
1. ImageMagick (Linux, macOS, Windows) 
ImageMagick is the industry-standard tool for command-line image manipulation. 2025 versions (v7+) use the magick command, while older versions use convert. 
Resize to a specific width (auto height):
magick input.jpg -resize 800x output.jpg
Resize by percentage:
magick input.jpg -resize 50% output.jpg
Force exact dimensions (ignore aspect ratio):
magick input.jpg -resize 300x200! output.jpg
Batch resize all images in a folder:
magick mogrify -resize 800x *.jpg
(Note: mogrify overwrites original files unless a different output path is specified.) 
2. SIPS (macOS Only)
macOS includes a native "Scriptable Image Processing System" that requires no installation. 
Resize keeping aspect ratio:
sips -Z 800 image.jpg
(The -Z flag sets the maximum dimension for both width and height.)
Batch resize a whole directory:
sips -Z 800 *.png --out resized_folder/ 
3. ffmpeg (Cross-platform) 
While primarily for video, ffmpeg is highly efficient for single or batch image resizing. 
Resize to a specific width:
ffmpeg -i input.jpg -vf scale=800:-1 output.jpg 
4. npm / Node.js Tools
If you already use Node.js, you can install lightweight CLI wrappers.
img-resizer-cli:
npm install -g img-resizer-cli
img-resizer-cli --file image.jpg --width 500 --output ./dist 


