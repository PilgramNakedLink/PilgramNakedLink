<?php
$target_dir = "uploads/";
if(count($_FILES))
{
    $datetime = time();

    if (isset($_FILES['csvFile'])) {
        $tmpFile = $_FILES['csvFile'];
        $contents = file_get_contents($tmpFile['tmp_name']);
        file_put_contents("uploads/" . $datetime  . '-' . $tmpFile['name'], $contents);
        echo "data uploaded...";
    }
    if (isset($_FILES['traceFile'])) {
        $tmpFile = $_FILES['traceFile'];
        $contents = file_get_contents($tmpFile['tmp_name']);
        file_put_contents("traces/" . $tmpFile['name'], $contents);    
        echo "trace uploaded...";
    }
}
else
{
    echo '403';
}
?>