<?php

	function dirToArray($dir) {
  
	   $result = array();
	   $contents = array();

	   $cdir = scandir($dir);
	   $cdir = array_slice($cdir, -50, 50);
	   foreach ($cdir as $key => $value)
	   {
	      if (!in_array($value,array(".","..", ".DS_Store")))
	      {
	         if (is_dir($dir . DIRECTORY_SEPARATOR . $value))
	         {
	            $result[$value] = dirToArray($dir . DIRECTORY_SEPARATOR . $value);
	         }
	         else
	         {
	         	$file= $dir . DIRECTORY_SEPARATOR . $value;
				$csv= file_get_contents($file);
				$array = array_map("str_getcsv", explode("\n", $csv));		
				$json = json_encode($array);

				$parts = explode("-", basename($value, ".csv"));
				$traceFile = "." . DIRECTORY_SEPARATOR . "traces" . DIRECTORY_SEPARATOR . $parts[1] . ".trace";
				$trace = file_get_contents($traceFile);

				$row = new stdClass();
				$row->value = $value;
				$row->contents = $array;
				$row->trace = explode("\n", $trace);
				$row->traceFile = $traceFile;			

	            $result[] = $row;
	         }
	      }
	   }
	  
	   return $result;
	}

	$directory = './uploads';
	$scanned_directory = dirToArray($directory);

	echo json_encode($scanned_directory);	

?>