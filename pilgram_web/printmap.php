<?php

ini_set('post_max_size', '32M');

if(isset($_POST['svgcontent']))
{
    $datetime = time();
    $filename = "svg-upload-" . $datetime . ".svg";
    file_put_contents("svguploads/" . $filename, $_POST['svgcontent']);
    shell_exec("./printmap.sh " . $filename);
    echo json_encode('ok');
}
else	
{
    echo '403';
}
?>