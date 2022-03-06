<!DOCTYPE html>
<html>
<head>
   <meta charset="utf-8">
   <title>Pilgram</title>

   <script src="js/jquery.js"></script>
   <script src="js/lodash.js"></script>

   <style type="text/css">
      body, html {
        background-color: #fff;
        margin: 0;
        min-with:  100%;
        min-height:  100%;
      }

      .radar {
        width: 1080px;
        height: 1080px;
        background-size: 1080px 1080px;
        position: fixed;
        left: 30%;
        top: 50%;
        transform: translate(-540px, -540px);
      }

      .levels {
       display: block;
       margin: auto;
       position: fixed;
       left: 30%;
       top: 50%;
       transform: translate(-673px, -519px) scale(1.2, 1.2);
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
      }

      .trace {
        top:  40px;
      }
   </style>
</head>
<body>
   <div class="">
      <div class="radar" id="radar" style="display: none">
        <div class="pointer"></div>
      </div>
      <svg id="radar-container" height="1300" width="1080" class="levels" style="background-color: transparent;" fill="none">
      </svg>
         <script type="text/javascript">
            var elem = document.body;
            function openFullscreen() {
              if (elem.requestFullscreen) {
                elem.requestFullscreen();
              } else if (elem.webkitRequestFullscreen) { /* Safari */
                elem.webkitRequestFullscreen();
              } else if (elem.msRequestFullscreen) { /* IE11 */
                elem.msRequestFullscreen();
              }
            }
      </script>
      <button id="fullscreen" onclick="openFullscreen()">Full Screen</button>
      <div class="content">
      </div>  
      <div class="trace">
      </div>
   </div>

   <script type="text/javascript">
     var pilgram = window.pilgram = {
        rings: 10
     };

     $(function() {
        // 
        pilgram.getFiles = () => {
          $.ajax({
            method: 'GET',
            url: './files.php',
            success: (response) => {
              pilgram.data = JSON.parse(response);
              _.each(pilgram.data, (trace) => {
                trace.filteredContents = _.filter(_.uniqBy(trace.contents, (row) => row[3]), (row) => row[5] == 'success');
              })
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

        var lastX, lastY;

        pilgram.drawLevel = (cleanRow, index, last) => {
          console.log(cleanRow);
          const parts = cleanRow[1].split('.');
          const reducer = (accumulator, currentValue) => parseInt(accumulator)   + parseInt(currentValue);
          const level = index;
          const amount = pilgram.rings;
          const maxRadius = 540;
          const increase = 350 / amount;
          const radius = level * increase;
          const angleG = parts.reduce(reducer) % 360;
          const angle = angleG * Math.PI / 180;

          console.log(angle);
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
        };

        pilgram.drawTree = () => {
          // this is the one
          var tree = pilgram.getGraph();
          pilgram.rings = 17;
          pilgram.createRings();
          var visited = {};
          var printNode = (key, node, level, angleG, parentX, parentY, indexInParent) => {
            console.log(key, level);  
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
          var content = document.getElementById('radar-container').outerHTML;

          $.ajax({
             method: 'post',
             url: 'printsvg.php',
             data: {
                svgcontent: content
             }
          });
        };

        pilgram.logTrace = (dataIndex) => {
         setTimeout(() => {
          $('#radar').fadeIn(1000);
          $('.trace').html(`<h3>Trace route</h3>`);
          $('.content').html(`<h3>Geo IP Data</h3>`);
          var timeout = 1700;
          var traceTimeout = 1200;
          pilgram.rings = pilgram.data[dataIndex].filteredContents.length;
          pilgram.createRings();
          _.each(pilgram.data[dataIndex].filteredContents, (row, index) => {
             setTimeout(() => {
               const cleanRow = Object.values(row).filter((obj) => obj != "")
               $('.content').append(`<p>${cleanRow.join(', ')}</p>`);

               pilgram.drawLevel(row, index, index == pilgram.data[dataIndex].filteredContents.length - 1);
             }, timeout);
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

        // var counter = 0;

        // setTimeout(() => {
        //   pilgram.logTrace(counter);
        // }, 2000);

        // var handler = setInterval(() => {
        //   pilgram.logTrace(counter);
        //   counter++;
        //   pilgram.getFiles();

        //   if (counter > pilgram.data.length) {
        //     clearInterval(handler);
        //   }
        // }, 120000);

     });
   </script>
</body>
</html>