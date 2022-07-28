Started following along with this video:
https://www.youtube.com/watch?v=Iapc-qGTEBQ&ab_channel=CodingTech

Decided to implement the async conn listener/broadcast listener differently than
the video.

start server:

``` sh
cargo run
```

From additinoal terminal windows, connect to server with:
``` sh
telnet localhost 8080
```

Type out a line and press enter, and the messages will show in the other
terminal windows.

There's a bug, something panics, I think it's related to client disconnects, not
sure.
