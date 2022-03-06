cd ../svgoutput/
inkscape -w 1024 -h 1024 $1 -o $1.png
lpr -P EPSON_LX_350 -o media=A4 -o sides=two-sided-long-edge -o InputSlot=tray-3 "$1.png"
