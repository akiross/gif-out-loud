use freetype::{Library, face::{Face, LoadFlag}, render_mode::RenderMode};
use gif::{Frame, Encoder, Repeat, SetParameter};
use std::fs::File;
use std::borrow::Cow;
use std::convert::TryFrom;

struct Script {
    frames: Vec<Vec<u8>>,
    sizes: Vec<(usize, usize)>,
    max_w: usize,
    max_h: usize,
}

fn parse_color(s: &str) -> Result<(u8, u8, u8), String> {
    if !s.starts_with("0x") {
        Err("Color does not start with 0x".to_owned())
    } else if s.len() != 8 {
        Err("Only 0xRRGGBB supported".to_owned())
    } else if !s[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        Err("Invalid hex digit found".to_owned())
    } else {
        let r = u8::from_str_radix(&s[2..4], 16).expect("Cannot parse u8");
        let g = u8::from_str_radix(&s[4..6], 16).expect("Cannot parse u8");
        let b = u8::from_str_radix(&s[6..8], 16).expect("Cannot parse u8");
        Ok((r, g, b))
    }
}

impl Script {
    fn from(text: &str, face: &Face) -> Self {
        let mut frames = vec![];
        let mut sizes = vec![];
        let mut max_w = 0;
        let mut max_h = 0;

        for c in text.chars() {
            let (w, h, frame) = {
                // Now we can take a character (whatever that means) and get its glyph index and this
                // is done using a charmap. The unicode charmap is used by default.
                let glyph_idx = face.get_char_index(c as usize);
                //let glyph_idx = face.get_char_index('a' as usize);
                // With the index, we can load the glyph into a glyph slot, where information about
                // it can be retrieved for usage (e.g. rendering). In this case, we load the glyph and
                // carry out no special operation (flag is DEFAULT).
                face.load_glyph(glyph_idx, LoadFlag::DEFAULT).expect("Cannot load glyph");
                // We can now render the glyph in the gliph slot, obtained via the gliph() method.
                let glyph = face.glyph();
                // 8 bit grayscale rendering is done in this case, and the buffer is accessible
                // via glyph.bitmap(). We want this to fail if rendering goes wrong.
                glyph.render_glyph(RenderMode::Normal).expect("Cannot render glyph");
                // Copy bytes to vector
                let w = glyph.bitmap().width() as usize;
                let h = glyph.bitmap().rows() as usize;
                
                if w == 0 || h == 0 {
                    // Ensure that frame has not size 0, some readers will fail
                    (1, 1, vec![0])
                } else {
                    let frame = {
                        let bit = glyph.bitmap();
                        (0..w*h).map(|i| bit.buffer()[i]).collect()
                    };
                    (w, h, frame)
                }
            };

            max_w = max_w.max(w);
            max_h = max_h.max(h);
            frames.push(frame);
            sizes.push((w, h));
        }

        Self { frames, sizes, max_w, max_h }
    }

    fn render_gif(&self, path: &str, fg_col: (u8, u8, u8), bg_col: (u8, u8, u8)) {
        let fg_r = fg_col.0 as usize;
        let fg_g = fg_col.1 as usize;
        let fg_b = fg_col.2 as usize;

        let bg_r = bg_col.0 as usize;
        let bg_g = bg_col.1 as usize;
        let bg_b = bg_col.2 as usize;

        // Build colormap
        let mut color_map = vec![0xff; 3 * 256];
        for (i, col) in color_map.chunks_mut(3).enumerate() {
            col[0] = u8::try_from(fg_r * i / 255 + bg_r * (255 - i) / 255).unwrap();
            col[1] = u8::try_from(fg_g * i / 255 + bg_r * (255 - i) / 255).unwrap();
            col[2] = u8::try_from(fg_b * i / 255 + bg_r * (255 - i) / 255).unwrap();
        }
        
        println!("Colormap is: {:?}", color_map);

        let mut image = File::create(path).unwrap();
        let mut encoder = Encoder::new(&mut image, self.max_w as u16, self.max_h as u16, color_map.as_slice()).unwrap();
        encoder.set(Repeat::Infinite).expect("Cannot set infinite repeat");

        for (i, f) in self.frames.iter().enumerate() {
            let mut frame = Frame::default();
            let (w, h) = self.sizes[i];

            println!("Rendering frame {} of size {}x{}", i, w, h);

            // TODO position should be a function which might vary in time to create "marquee"
            // effect. It might make the text more readable, even if using more frames
            frame.left = ((self.max_w - w) / 2) as u16;
            frame.top = ((self.max_h - h) / 2) as u16;
            frame.width = w as u16;
            frame.height = h as u16;
            frame.buffer = Cow::Borrowed(f);
            frame.transparent = Some(0); // Transparent color
            frame.dispose = gif::DisposalMethod::Background;
            frame.delay = 30;
            encoder.write_frame(&frame).expect("Unable to write frame");
        }
    }
}

fn main() {
    // In this example code we'll draw onto a bitmap a string with a given font face.
    // We expect the string and the face as arguments of this program:
    let text = std::env::args().nth(1).expect("Pls provide text to render");
    // Second is the foreground color
    let fg_color = std::env::args().nth(2).unwrap_or(String::from("0xffffff"));
    let fg_color = parse_color(fg_color.as_str()).expect("Cannot parse fg color");
    // Third is background color
    let bg_color = std::env::args().nth(3).unwrap_or(String::from("0xffffff"));
    let bg_color = parse_color(bg_color.as_str()).expect("Cannot parse bg color");
    // Third is the face, but fall back to an open-source default font.
    let face = std::env::args().nth(4).unwrap_or(String::from("LiberationSans-Bold.ttf"));

    println!("Text to print: {}", text);
    println!("Font to use: {}", face);
    println!("Colors to use: {:?} {:?}", fg_color, bg_color);

    // To draw some text with freetype, initialize the library (FT_Init_FreeType) first.
    // If this doesn't succeed it's Ok to panic, so we unwrap.
    let lib = Library::init().unwrap();
    // Then we load the font face: the second argument is the face index if there
    // are more font faces per file and 0 is the first one (always present).
    // Again, if we cannot create a new face, fail loudly.
    let face = lib.new_face(face, 0).unwrap();
    // Let's see how many glyphs are present in this face:
    println!("Glyphs count: {}", face.raw().num_glyphs);
    // Now we set the character size. Assuming we are on a 96dpi device, we want
    // a 16pt character size. A point is a 1/72th of inch, and char width and height
    // are expressed as 1/64th of a point.
    // Here width is 16 * 64 * 1/64 = 16 points, height is 0 meaning "same as width".
    face.set_char_size(32 * 64, 0, 96, 96).expect("Unable to set font size");
    
    let s = Script::from(&text, &face);

    s.render_gif("prova.gif", fg_color, bg_color);
}
