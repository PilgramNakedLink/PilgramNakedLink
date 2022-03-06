<!DOCTYPE html>
<html>
<head>
   <meta charset="utf-8">
   <title>Pilgram</title>

   <script src="js/jquery.js"></script>
   <script src="js/lodash.js"></script>
   <script src="js/moment.js"></script>

</head>
<body>

    <style type="text/css" id="styles">
      body, html {
        background-color: #fff;
        margin: 0;
        min-with:  100%;
        min-height:  100%;
      }

      .container {
        background-color: transparent;
        min-width: 100%;
        min-height: 100%;
        position: absolute;
      }

      .radar {
        width: 1080px;
        height: 1080px;
        background-size: 1080px 1080px;
        position: fixed;
        left: 30%;
        top: 50%;
        transform: translate(-540px, -540px);
        z-index: 999;
      }

      .levels {
       display: block;
       margin: auto;
       position: fixed;
       left: 30%;
       top: 50%;
       transform: translate(-673px, -519px) scale(1.2, 1.2);
       z-index: 999;
      }

      .radar:hover {
        background: none;
      }

      .radar .pointer {
        position: fixed;
        z-index: 1024;
        left: 10.5820106%;
        right: 10.5820106%;
        top: 10.5820106%;
        bottom: 50%;
        will-change: transform;
        transform-origin: 50% 100%;
        border-radius: 50% 50% 0 0 / 100% 100% 0 0;
        background-image: linear-gradient(135deg, 
          rgba(185, 185, 185, 0.5) 0%, 
          rgba(0, 0, 0, 0.02) 70%,
          rgba(0, 0, 0, 0) 100%
          );
        clip-path: polygon(100% 0, 
          100% 10%,ß
          50% 100%, 0 100%, 0 0);
        
        animation: rotate360 3s infinite linear;
      }

      .radar .pointer:after {
        content: "";
        position: absolute;
        width: 50%;
        bottom: -1px;
        border-top: 2px solid rgba(255, 255, 255, 0.8);
        box-shadow: 0 0 3px rgba(255, 255, 255, 0.6);
        border-radius: 9px;
        z-index: 1000;
      }

      .shadow {
        position: absolute;
        left: 11%;
        top: 11%;
        right: 11%;
        bottom: 11%;
        margin: auto;
        border-radius: 9999px;
        box-shadow: 0 0 66px 6px #aaaaaa;
        animation: shadow 1s infinite ease;
      }


      @keyframes rotate360 {
        0% {
          transform: rotate(0deg);
        }
        to {
          transform: rotate(-360deg);
        }
      }

      @keyframes shadow {
        0% {
          opacity: 0;
        }
        50% {
          opacity: 1;
        }
        to {
          opacity: 0;
        }
      }

      #fullscreen {
         position: fixed;
         left: 40px;
         top: 40px;
         z-index: 999999;
      }

      .content, .trace {
         position: fixed;
         right:  40px;
         top: 53%;
         max-height: 40%;
         height: auto;
         overflow: auto;
         color: #222;
         line-height: 11px;
         font-size: 10px;
         font-family: 'Courier New';
         width: 35%;
         z-index:  999;
      }

      .trace {
        top:  40px;
      }

      .tiles {
        position: absolute;
        width: 100%;
        height: 100%;
        margin: 0;
          /* background: red; */
      }

      .tile {
        width: 20%;
        height: 33.33%;
        float: left;
        margin: 0;
        padding: 0;
        background-position: center;
        background-size: cover;
    }

    .radar-status {
      position: fixed;
      z-index:  9999;
      left: 40px;
      bottom: 30px;
      font-family: Courier New, monospace;
      font-size: 10px;
    }

    .status {
      border-radius: 50%;
      border: 5px solid;
      width: 1px;
      height: 1px;
      position: absolute;
      left: -15px;
      top:  -1px;
    }

    .offline {
      border-color:  red;
    }

    .online {
      border-color:  green;
    }

    .blink {
      animation: blinker 1s linear infinite;
    }

    @keyframes blinker {
      50% {
        opacity: 0;
      }
    }

   </style>

   <button id="fullscreen"onclick="window.openFullscreen()">Full Screen</button>

   <div class="container">
      <svg width="1920" height="1080" class="tiles" id="tiles"></svg>
      <div class="radar" id="radar" style="display: none">
        <div class="pointer"></div>
      </div>
      <svg id="radar-container" height="1300" width="1080" class="levels" style="background-color: transparent;" fill="none"></svg>
      <svg id="radar-container-print" width="7680" height="4320" class="levels-print" style="background-color: transparent;" fill="none">
      </svg>
         <script type="text/javascript" id="fullscreen">
            var elem = document.body;

            window.openFullscreen = () => {
              $('#fullscreen').hide();
              if (elem.requestFullscreen) {
                elem.requestFullscreen();
              } else if (elem.webkitRequestFullscreen) { /* Safari */
                elem.webkitRequestFullscreen();
              } else if (elem.msRequestFullscreen) { /* IE11 */
                elem.msRequestFullscreen();
              }
            }
      </script>
      <div class="content">
      </div>  
      <div class="trace">
      </div>
      <div class="radar-status" id="radar-status">
        <span id="data-status" class="status offline blink"></span> Showing offline data
      </div>
   </div>

   <script src="https://maps.google.com/maps/api/js?libraries=geometry&sensor=false&key=AIzaSyAeJm2w3wL01ctB_0MjK2edSotxvS9EJ0s"></script>

  <script type="text/javascript">
     // printing time interval (half of this time per map/radar) in milliseconds
     // 10 minutes
     
     var TRACE_INTERVAL = <?php echo file_get_contents('TRACE_INTERVAL')?> * 60 * 1000;
     var PRINT_INTERVAL = <?php echo file_get_contents('PRINT_INTERVAL')?> * 60 * 1000;
     var counter = 0;
     var onlineData = false;
     var geocoder = new google.maps.Geocoder();
     var pilgram = window.pilgram = {
        rings: 10,
        location: {}
     };

     var setOnlineStatus = (status) => {
        console.log("setting status to: " + status);
        onlineData = status;

        if (onlineData) {
          $("#radar-status").hide();
        } else {
          $("#radar-status").show();
        }
     };

     var verifyOnlineStatus = (trace, index) => {
        const currentFileDate = moment(new Date(parseInt(trace.value.split('-')[0]) * 1000));
        const currentDate = moment();
        const difference = currentDate.diff(currentFileDate, 'minutes');

        console.log( difference );

        if (difference < 20 && !onlineData) {
          counter = index;

          setOnlineStatus(true);
        }
     };

     $(function() {
        // 
        pilgram.getFiles = () => {
          $.ajax({
            method: 'GET',
            url: './files.php',
            success: (response) => {
              pilgram.data = JSON.parse(response);
              _.each(pilgram.data, (trace, index) => {
                trace.filteredContents = _.filter(_.uniqBy(trace.contents, (row) => row[3]), (row) => row[5] == 'success');

                // determine the status of the data
                verifyOnlineStatus(trace, index);
              });
            }
          });
        };

        pilgram.getIpTrace = (index) => {
          var ips = _.map(pilgram.data[index].trace, (trace) => trace.split(' ')[1])
          var ipsf = _.filter(ips, (v) => v && v.indexOf("*") == -1);
          var ipsf = _.filter(ipsf, (v) => v && v.indexOf("X") == -1);
          console.log(ipsf);
          return ipsf;
        };
    
        pilgram.getGraph = () => {
          var graph = {};
          for (var index = 0; index < pilgram.data.length; index++) {
            var traces = pilgram.getIpTrace(index);
            traces.forEach((ip, index) => {
              if (index == 0) {
                graph[ip] = { parent: "root", branches: {}}
              } else {
                if (!graph[ip]) {
                   graph[ip] = { parent: "", branches: {}}
                }
                graph[ip].parent = traces[index-1];
                if (!graph[traces[index-1]].branches) {
                   graph[traces[index-1]].branches = {};
                }
                graph[traces[index-1]].branches[traces[index]] = true;
                if (!graph[traces[index]]) {
                  graph[traces[index]] = { parent: "", branches: {}};
                }
              }
            });
          } 

          return graph;
        };

        pilgram.getLocation = (address, callback) => {
           if (geocoder) {
              geocoder.geocode({ 'address': address }, function (results, status) {
                  console.log("Geocoding results:", results);
                 if (status == google.maps.GeocoderStatus.OK) {
                    const zoom = Math.floor(Math.random() * 5) + 11;
                    callback(null, 
                      { 
                        data: results[0].geometry,
                        image: `https://maps.googleapis.com/maps/api/staticmap?center=${results[0].geometry.location.lat()},${results[0].geometry.location.lng()}&zoom=14&size=1200x1200&maptype=satellite&key=AIzaSyAeJm2w3wL01ctB_0MjK2edSotxvS9EJ0s`
                      });
                 }
                 else {
                    callback("error getting that location.");
                 }
              });
           }
        };

        pilgram.reverseLocation = (lat, lng, callback) => {

        };

        pilgram.computeHeading = (pointA, pointB) => {
            console.log("computing heading");
            console.log(pointA, pointB);
            const pa = {
              lat: pointA.lat(),
              lng: pointA.lng()
            };
            const pb = {
              lat: pointB.lat(),
              lng: pointB.lng()
            };
            console.log(pa);
            const heading = google.maps.geometry.spherical.computeHeading(
              pa,
              pb
            );

            return heading;
        };

        pilgram.getLocalLocation = (success, error) => {
          if(navigator.geolocation) {
            navigator.geolocation.getCurrentPosition(success, error);
          } else {
            alert("Geolocation is not supported by this browser.");
          }
        }

        pilgram.createRingsPrint = () => {
          const amount = pilgram.rings;
          var maxRadius = 1080 * 2;
          var maxX = 1920 * 4;
          var maxY = 1080 * 4;
          var increase = maxRadius / amount;
          var radius = 1;

          document.querySelector('#radar-container-print').innerHTML = "";

          for (var ring = 1; ring <= amount; ring++) {
            setTimeout(() => {
              document.querySelector('#radar-container-print').innerHTML += `<circle cx="${maxX / 2}" cy="${maxY / 2}" r="${radius}" stroke="#888" stroke-width="2" fill="none" />`;
              radius += increase;
            }, 60 * ring);
          }
        };

        pilgram.createRings = () => {
          const amount = pilgram.rings;
          var maxRadius = 350;
          var increase = maxRadius / amount;
          var radius = 1;

          document.querySelector('#radar-container').innerHTML = "";

          for (var ring = 1; ring <= amount; ring++) {
            setTimeout(() => {
              document.querySelector('#radar-container').innerHTML += `<circle cx="${650}" cy="${540}" r="${radius}" stroke="#888" stroke-width="1" fill="none" />`;
              radius += increase;
            }, 60 * ring);
          }
        };

        var lastX, lastY, lastLocation;
        var lastXp, lastYp, lastLocationp;

        pilgram.drawLevelPrint = (cleanRow, index, last, timeout) => {
          setTimeout(() => {
            pilgram.getLocation(`${cleanRow[10]}, ${cleanRow[12]},`, (err, result) => {

              var headingAngle = null;
              var currentLocation = result && result.data ? result.data.location : null;
              var currentTile = result && result.image ? result.image : null;
              var size = 384;

              if (currentTile) {
                if (mapWindow) {
                  (async function(last) {
                    let blob = await fetch(currentTile).then(r => r.blob());
                    let dataUrl = await new Promise(resolve => {
                      let reader = new FileReader();
                      reader.onload = () => resolve(reader.result);
                      reader.readAsDataURL(blob);
                    });
                    // now do something with `dataUrl`
                    console.log(`index=${index} x="${((index -1) % 5) * size}" y="${(Math.floor((index -1) / 5)) * size}"`);
                    mapWindow.document.querySelector('#tiles').innerHTML += `<image width="${size}" height="${size}" x="${((index -1) % 5) * size}" y="${(Math.floor((index -1) / 5)) * size}" xlink:href="${dataUrl}" />`;

                  })(last);
                }
              }

              if ((pilgram.location || lastLocationp) && currentLocation) {
                headingAngle = pilgram.computeHeading(currentLocation, pilgram.location || lastLocationp);
              }

              const level = index;
              const amount = pilgram.rings;
              const maxRadius = 1080 * 2;
              const increase = maxRadius / amount;
              const radius = level * increase;
              let angleG = 0;
              const halfX = 1920 * 2;
              const halfY = 1080 * 2;

              if (headingAngle > 0) {
                angleG = headingAngle;
              } else if (headingAngle < 0) {
                angleG = 360 + headingAngle;
              }

              const angle = angleG * Math.PI / 180;

              var x = radius * Math.sin(angle);
              var y = radius * Math.cos(angle); 

              const showText = `${cleanRow[6]} (${cleanRow[10]}, ${cleanRow[12]}, ${cleanRow[21]}, ${cleanRow[22]})`;
              let fill = '#aa2222';
              let r = 12;

              if (last) {
                  r = 18;
                  fill = '#77cc98'
              }

              document.querySelector('#radar-container-print').innerHTML += `<circle 
                                                                          id="ip-${cleanRow[6]}"
                                                                          class=".level-${index}"
                                                                          cx="${halfX + x}" 
                                                                          cy="${halfY + y}"
                                                                          r="${r}"
                                                                          stroke="white"
                                                                          stroke-width="2"
                                                                          fill="${fill}">
                                                                      </circle>`;
              if (index > 0) {
                document.querySelector('#radar-container-print').innerHTML += `<text
                                                                            id="text-${cleanRow[6]}"
                                                                            class=".level-${index}"
                                                                            style="font: italic 40px sans-serif"
                                                                            x="${halfX + x + 5}"
                                                                            y="${halfY + y - 5}" 
                                                                            fill="#222"
                                                                            transform="rotate(${0} ${halfX + x + 3},${halfY + y - 5})">
                                                                              ${showText}
                                                                            </text>`;
                const currentX = halfX + x;
                const currentY = halfY + y;

                document.querySelector('#radar-container-print').innerHTML += `<line x1="${lastXp}" y1="${lastYp}" x2="${currentX}" stroke-width="2" y2="${currentY}" stroke="#999" />`;
              } else {
                document.querySelector('#radar-container-print').innerHTML += `<text
                                                                            id="text-root"
                                                                            class=".level-${index}"
                                                                            style="font: italic 40px sans-serif"
                                                                            x="${halfX + x + 5}"
                                                                            y="${halfY + y - 5}" 
                                                                            fill="#222"
                                                                            transform="rotate(${0} ${halfX + x},${halfY + y})">
                                                                              ${cleanRow[0]} (root)
                                                                            </text>`;
              }

              lastXp = halfX + x;
              lastYp = halfY + y;
              lastLocationp = currentLocation;

              if (last) {
                pilgram.queuePrint(); 
              }
            });
          }, timeout);
        };

        pilgram.drawLevel = (cleanRow, index, last, timeout) => {
          setTimeout(() => {
            pilgram.getLocation(`${cleanRow[10]}, ${cleanRow[12]},`, (err, result) => {

              var headingAngle = null;
              var currentLocation = result && result.data ? result.data.location : null;
              var currentTile = result && result.image ? result.image : null;
              var size = 384;

              if (currentTile) {
                if (mapWindow) {
                  (async function(last) {
                    let blob = await fetch(currentTile).then(r => r.blob());
                    let dataUrl = await new Promise(resolve => {
                      let reader = new FileReader();
                      reader.onload = () => resolve(reader.result);
                      reader.readAsDataURL(blob);
                    });
                    // now do something with `dataUrl`
                    console.log(`index=${index} x="${((index -1) % 5) * size}" y="${(Math.floor((index -1) / 5)) * size}"`);
                    mapWindow.document.querySelector('#tiles').innerHTML += `<image width="${size}" height="${size}" x="${((index -1) % 5) * size}" y="${(Math.floor((index -1) / 5)) * size}" xlink:href="${dataUrl}" />`;


                    if (last) {
                      setTimeout(pilgram.queuePrintMap, PRINT_INTERVAL / 2);
                    }

                  })(last);
                }
              }

              if ((pilgram.location || lastLocation) && currentLocation) {
                console.log(currentLocation);
                headingAngle = pilgram.computeHeading(currentLocation, pilgram.location || lastLocation);
              }

              const level = index;
              const amount = pilgram.rings;
              const maxRadius = 540;
              const increase = 350 / amount;
              const radius = level * increase;
              let angleG = 0;

              if (headingAngle > 0) {
                angleG = headingAngle;
              } else if (headingAngle < 0) {
                angleG = 360 + headingAngle;
              }

              const angle = angleG * Math.PI / 180;

              var x = radius * Math.sin(angle);
              var y = radius * Math.cos(angle); 

              const showText = `${cleanRow[6]} (${cleanRow[10]}, ${cleanRow[12]}, ${cleanRow[21]}, ${cleanRow[22]})`;
              let fill = '#aa2222';
              let r = 3;

              if (last) {
                  r = 5;
                  fill = '#77cc98'
              }

              document.querySelector('#radar-container').innerHTML += `<circle 
                                                                          id="ip-${cleanRow[6]}"
                                                                          class=".level-${index}"
                                                                          cx="${650 + x}" 
                                                                          cy="${540 + y}"
                                                                          r="${r}"
                                                                          stroke="white"
                                                                          stroke-width="1"
                                                                          fill="${fill}">
                                                                      </circle>`;
              if (index > 0) {
                document.querySelector('#radar-container').innerHTML += `<text
                                                                            id="text-${cleanRow[6]}"
                                                                            class=".level-${index}"
                                                                            style="font: italic 8px sans-serif"
                                                                            x="${650 + x + 5}"
                                                                            y="${540 + y - 5}" 
                                                                            fill="#222"
                                                                            transform="rotate(${0} ${650 + x + 3},${540 + y - 5})">
                                                                              ${showText}
                                                                            </text>`;
                const currentX = 650 + x;
                const currentY = 540 + y;

                document.querySelector('#radar-container').innerHTML += `<line x1="${lastX}" y1="${lastY}" x2="${currentX}" y2="${currentY}" stroke="#999" />`;
              } else {
                document.querySelector('#radar-container').innerHTML += `<text
                                                                            id="text-root"
                                                                            class=".level-${index}"
                                                                            style="font: italic 8px sans-serif"
                                                                            x="${650 + x + 5}"
                                                                            y="${540 + y - 5}" 
                                                                            fill="#222"
                                                                            transform="rotate(${0} ${650 + x},${540 + y})">
                                                                              ${cleanRow[0]} (root)
                                                                            </text>`;
              }

              lastX = 650 + x;
              lastY = 540 + y;
              lastLocation = currentLocation;
            });
          }, timeout);
        };


        pilgram.drawTree = () => {
          // this is the one
          var tree = pilgram.getGraph();
          pilgram.rings = 17;
          pilgram.createRings();
          pilgram.createRingsPrint();
          var visited = {};
          var printNode = (key, node, level, angleG, parentX, parentY, indexInParent) => {
            if (level >= Object.keys(tree).length) return;

            var nodeName = 'node-' + key.replace(/\./g, '-');
   
            const maxRadius = 540;
            const increase = 350 / 17;
            const radius = level * increase;

            const piece = 30 / (level + 1);
            if (indexInParent == 0) {
              threshold = piece;
            } else if (indexInParent % 2 == 0) {
              threshold = piece * (indexInParent + 1);
            } else {
              threshold = -1 * piece * indexInParent;
            }

            const angle = (angleG + threshold) * Math.PI / 180;

            var x = 650 + radius * Math.sin(angle);
            var y = 540 + radius * Math.cos(angle);

            if ($(`#${nodeName}`).length == 0){
                document.querySelector('#radar-container')
                      .innerHTML += `<circle
                                        id="${nodeName}"
                                        data-x="${x}"
                                        data-y="${y}"
                                        cx="${x}" 
                                        cy="${y}"
                                        r="${3}"
                                        stroke="white"
                                        stroke-width="1"
                                        fill="#aaff56">
                                     </circle>`;

                document.querySelector('#radar-container').innerHTML += `<text
                                        style="font: italic 8px sans-serif"
                                        x="${x + 5}"
                                        y="${y - 5}" 
                                        fill="#222"
                                        transform="rotate(${0} ${x + 3},${y - 5})">
                                          ${key}
                                        </text>`;
                document
                  .querySelector('#radar-container')
                  .innerHTML += `<line x1="${parentX}" y1="${parentY}" x2="${x}" y2="${y}" stroke="#999" stroke-width="1"/>`;
            }

            if (Object.keys(node.branches).length == 0) return;

            Object.keys(node.branches).forEach((key, index) => {
              printNode(key, tree[key], level + 1, angleG, x, y, index);
            });
          }

          setTimeout(() => {
             printNode("10.1.10.1", tree["10.1.10.1"], 0, 76, 650, 540, 0);
          }, 3000);
        };

        pilgram.queuePrint = () => {
          var content = document.getElementById('radar-container-print').outerHTML;

          $.ajax({
             method: 'post',
             url: 'printsvg.php',
             data: {
                svgcontent: content
             }
          });
        };

        pilgram.queuePrintMap = () => {
          var content = mapWindow.document.getElementById('tiles').outerHTML;

          $.ajax({
             method: 'post',
             url: 'printmap.php',
             data: {
                svgcontent: content
             }, error: (error) => {
              console.log(error);
             }, success: (res) => {
              console.log(res);
             }
          });
        };

        pilgram.logTrace = (dataIndex) => {
         setTimeout(() => {
          $('#radar').fadeIn(1000);
          $('.trace').html(`<h3>Trace route (${pilgram.data[dataIndex].value})</h3>`);
          $('.content').html(`<h3>Geo IP Data</h3>`);
          $('.tiles').html(``);
          if (mapWindow) {
            mapWindow.document.body.querySelector('#tiles').innerHTML = "";
          }

          // verify status before rendering
          verifyOnlineStatus(pilgram.data[dataIndex], dataIndex);

          var timeout = 1700;
          var traceTimeout = 1200;
          pilgram.rings = pilgram.data[dataIndex].filteredContents.length;
          pilgram.createRings();
          pilgram.createRingsPrint();

          _.each(pilgram.data[dataIndex].filteredContents, (row, index) => {
             const cleanRow = Object.values(row).filter((obj) => obj != "")
             $('.content').append(`<p>${cleanRow.join(', ')}</p>`);

             pilgram.drawLevel(row, index, index == pilgram.data[dataIndex].filteredContents.length - 1, timeout);
             pilgram.drawLevelPrint(row, index, index == pilgram.data[dataIndex].filteredContents.length - 1, timeout);
             timeout += 1200;
          });

          _.each(pilgram.data[dataIndex].trace, (row) => {
             setTimeout(() => {
               $('.trace').append(`<p>${row}</p>`);
             }, traceTimeout);
             traceTimeout += 1300;
          });
         }, 3000);
        };

        // get the data
        pilgram.getFiles();

        pilgram.getLocalLocation((position) => {
            pilgram.location = {
              lat: () => position.coords.latitude,
              lng: () => position.coords.longitude
            };

            setTimeout(() => {
              pilgram.logTrace(counter);
              counter++;
            }, 2000);

            const processTrace = () => {
              pilgram.logTrace(counter);
              counter++;
              console.log("index=", counter);
              console.log("data.length=", pilgram.data.length);
              pilgram.getFiles();

              if (counter >= pilgram.data.length) {
                // restart to show offline data if it gets to the end and there's no new data...
                counter = 0;
                setOnlineStatus(false);
              };
            }

            var handler = setInterval(processTrace, TRACE_INTERVAL);
            
          }, (error) => {
            console.log(error);
          } 
        );

        window.mapWindow = null;

        pilgram.openMap = () => {
          mapWindow = window.open("", "MAP", "toolbar=0,location=0,menubar=0");
          mapWindow.document.body.innerHTML = document.body.querySelector('.tiles').outerHTML;
          mapWindow.document.body.innerHTML += document.body.querySelector('#styles').outerHTML;
          $('.tiles').hide();
        };

        setTimeout(pilgram.openMap, 500);

        pilgram.saveMap = () => {
          var canvas = document.createElement("canvas");
          context = canvas.getContext('2d');

          make_base();

          function make_base()
          {
            base_image = new Image();
            base_image.src = 'img/base.png';
            base_image.onload = function(){
              context.drawImage(base_image, 100, 100);
            }
          }
        };

     });
   </script>
</body>
</html>