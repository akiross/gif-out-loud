# gif-out-loud

GIF maker to say things.

Usage examples (none of this is working right now)

    # Produces a white-on-transparent GIF with one letter per frame, 250ms each
    cargo run -- "Hello, World"

    # Change foreground to green
    cargo run -- --fore 0x00FF00 "Hello, World"

    # Change background to black
    cargo run -- --back 0x000000 "Hello, World"

    # Change delay
    cargo run -- --delay 500 "Hello, World"

    # Specify font
    cargo run -- --face "myfont.ttf" "Hello, World"
