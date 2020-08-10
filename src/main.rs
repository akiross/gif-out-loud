use freetype::{Library, face::LoadFlag};
use gif::{Frame, Encoder, Repeat, SetParameter};
use std::fs::File;
use std::borrow::Cow;
 
fn main() {
    let lib = Library::init().unwrap();
    // Load a font face
    let face = lib.new_face("LiberationSans-Bold.ttf", 0).unwrap();
    //let face = lib.new_face("TwitterColorEmoji-SVGinOT.ttf", 0).unwrap();

    // Set the font size
    face.set_char_size(40 * 64, 0, 50, 0).unwrap();

    // String to write
    let s = "Hello, World";
    let gif_path = "prova.gif";

    let mut max_w = 0;
    let mut max_h = 0;

    let bitmaps = s.chars().map(|c| {
        // Load a character
        face.load_char(c as usize, LoadFlag::RENDER).unwrap();
        // Get the glyph instance
        let glyph = face.glyph();

        // Get bitmap
        let bitmap = glyph.bitmap();
        max_w = i32::max(max_w, bitmap.width());
        max_h = i32::max(max_h, bitmap.rows());
        let x = glyph.bitmap_left();
        let y = glyph.bitmap_top();
        println!("{} {}, {} {}, Pixel mod: {:?}", x, y, max_w, max_h, bitmap.pixel_mode());

        // Save with size
        bitmap
    }).collect::<Vec<_>>();

    println!("Font bitmap size {} {}", max_w, max_h);

    // R, G, B, ...
    let mut color_map = vec![0xff; 3 * 256]; // &[0xFF, 0xFF, 0xFF, 0, 0, 0];
    for i in 0..color_map.len() {
        color_map[i / 3] = (i / 3) as u8;
    }
    let (width, height) = (max_w as u16, max_h as u16);

    let mut image = File::create(gif_path).unwrap();
    let mut encoder = Encoder::new(&mut image, width, height, color_map.as_slice()).unwrap();
    encoder.set(Repeat::Infinite).unwrap();

    bitmaps.iter().for_each(|bitmap| {
        let mut frame = Frame::default();
        frame.width = width;
        frame.height = height;
        frame.buffer = Cow::Borrowed(bitmap.buffer());
        encoder.write_frame(&frame).unwrap();
    });
}
