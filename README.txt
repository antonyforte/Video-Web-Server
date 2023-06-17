Before testing the server first:

1 - Create a directory named static. and add videos to this directory
2 - In html/Files.html behind line 164(div class="return") modify the link and add your video links. example:

in the code is:
<div class="video-card">
  <div class="video-title">SummerTime Render EPS 2</div>
  <button onclick="openVideoPlayer('/static/summertime-render-episode-2-sub.mp4', 'SummerTime Render EPS 2')">Assistir</button>
  <button><a href="static/summertime-render-episode-2-sub.mp4" download>Baixar</a></button>
</div>

change it to:
<div class="video-card">
  <div class="video-title">MEU VIDEO</div>
  <button onclick="openVideoPlayer('/static/meuvídeo.mp4', 'meu vídeo')">Assistir</button>
  <button><a href="static/meuvídeo.mp4" download>Baixar</a></button>
</div>

Doing this i think you will not have any problems with the server :D
