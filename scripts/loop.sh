./target/release/tracer init
while :
do
  while read p; do
    echo "Exporting $p..."
    ./target/release/tracer trace $p > ./output/$p.trace
    echo "Writing contents..."  
    ./target/release/tracer export $p | tee ./export/$p.csv > ./output/$p.export
    echo "Uploading data..."
    curl -F "csvFile=@/Users/pilgram/Documents/pilgram_naked_link/export/$p.csv" 'http://localhost:8888/pilgram/upload.php'
    echo "Uploading trace..."
    curl -F "traceFile=@/Users/pilgram/Documents/pilgram_naked_link/output/$p.trace" 'http://localhost:8888/pilgram/upload.php'
    echo "\n"
  done <ip-range.txt
done
