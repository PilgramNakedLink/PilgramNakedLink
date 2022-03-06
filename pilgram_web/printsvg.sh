cd ./svguploads/
/Applications/Inkscape.app/Contents/MacOS/inkscape -d 300 -w 7680 -h 4320 $1 -o $1.png
lpoptions -o ColorMode=Black
lpr -P "HP_DesignJet_T650_24_in__FFD9CB__20210714152901" -o ColorMode=Black -o media=Custom.24x24in "$1.png"
