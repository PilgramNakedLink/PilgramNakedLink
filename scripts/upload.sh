find ./export -type f -print0 | xargs -0 ls -t | while read file
do
    echo "$file" # or whatever you want with $file, which may have spaces
               # so always enclose it in double quotes
done
