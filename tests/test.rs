extern crate bmfont;

use bmfont::{BMFont, CharPosition, OrdinateOrientation, Rect};
use std::fs::File;

const RUST_WORD: &'static str = "Rust";
const UNDERSCORE_CHARACTER: &'static str = "_";
const YOU_WORD: &'static str = "You";

fn create_bmfont(ordinate_orientation: OrdinateOrientation) -> BMFont {
    let file = File::open("font.fnt").unwrap();
    BMFont::new(file, ordinate_orientation).unwrap()
}

fn parse(s: &str, ordinate_orientation: OrdinateOrientation) -> Vec<CharPosition> {
    let font = create_bmfont(ordinate_orientation);
    let parse = font.parse(s);

    #[cfg(feature = "parse-error")]
    let parse = parse.unwrap();

    parse.collect()
}

fn assert_rect_equal(rect: &Rect, another_rect: &Rect) {
    assert_eq!(rect.x, another_rect.x);
    assert_eq!(rect.y, another_rect.y);
    assert_eq!(rect.max_x(), another_rect.max_x());
    assert_eq!(rect.max_y(), another_rect.max_y());
}

fn assert_char_positions_equal(char_position: &CharPosition, another_char_position: &CharPosition) {
    assert_rect_equal(&char_position.page_rect, &another_char_position.page_rect);
    assert_rect_equal(
        &char_position.screen_rect,
        &another_char_position.screen_rect,
    );
    assert_eq!(char_position.page_index, another_char_position.page_index);
}

fn create_char_position(page_rect: Rect, screen_rect: Rect) -> CharPosition {
    CharPosition {
        page_rect: page_rect,
        screen_rect: screen_rect,
        page_index: 0,
    }
}

fn page_rect_for_underscore() -> Rect {
    Rect {
        x: 221,
        y: 430,
        width: 44,
        height: 7,
    }
}

fn page_rect_for_capital_y() -> Rect {
    Rect {
        x: 222,
        y: 114,
        width: 50,
        height: 54,
    }
}

fn page_rect_for_capital_r() -> Rect {
    Rect {
        x: 147,
        y: 269,
        width: 48,
        height: 54,
    }
}

fn page_rect_for_u() -> Rect {
    Rect {
        x: 2,
        y: 458,
        width: 32,
        height: 41,
    }
}

fn page_rect_for_o() -> Rect {
    Rect {
        x: 2,
        y: 372,
        width: 37,
        height: 41,
    }
}

fn page_rect_for_s() -> Rect {
    Rect {
        x: 2,
        y: 415,
        width: 33,
        height: 41,
    }
}

fn page_rect_for_t() -> Rect {
    Rect {
        x: 37,
        y: 415,
        width: 21,
        height: 54,
    }
}

fn page_rects_for_rust_word() -> Vec<Rect> {
    vec![
        page_rect_for_capital_r(),
        page_rect_for_u(),
        page_rect_for_s(),
        page_rect_for_t(),
    ]
}

fn page_rects_for_you_word() -> Vec<Rect> {
    vec![
        page_rect_for_capital_y(),
        page_rect_for_o(),
        page_rect_for_u(),
    ]
}

fn screen_rect_for_capital_r_in_rust_word(y: i32) -> Rect {
    Rect {
        x: 6,
        y: y,
        width: 48,
        height: 54,
    }
}

fn screen_rect_for_u_in_rust_word(y: i32) -> Rect {
    Rect {
        x: 57,
        y: y,
        width: 32,
        height: 41,
    }
}

fn screen_rect_for_s_in_rust_word(y: i32) -> Rect {
    Rect {
        x: 94,
        y: y,
        width: 33,
        height: 41,
    }
}

fn screen_rect_for_t_in_rust_word(y: i32) -> Rect {
    Rect {
        x: 129,
        y: y,
        width: 21,
        height: 54,
    }
}

fn screen_rects_for_rust_word(ys: [i32; 4]) -> Vec<Rect> {
    vec![
        screen_rect_for_capital_r_in_rust_word(ys[0]),
        screen_rect_for_u_in_rust_word(ys[1]),
        screen_rect_for_s_in_rust_word(ys[2]),
        screen_rect_for_t_in_rust_word(ys[3]),
    ]
}

fn screen_rect_for_underscore(y: i32) -> Rect {
    Rect {
        x: -1,
        y: y,
        width: 44,
        height: 7,
    }
}

fn screen_rect_for_capital_y_in_you_word(y: i32) -> Rect {
    Rect {
        x: 0,
        y: y,
        width: 50,
        height: 54,
    }
}

fn screen_rect_for_o_in_you_word(y: i32) -> Rect {
    Rect {
        x: 0 + 48 + 2 - 7,
        y: y,
        width: 37,
        height: 41,
    }
}

fn screen_rect_for_u_in_you_word(y: i32) -> Rect {
    Rect {
        x: 0 + 48 - 7 + 40 + 5,
        y: y,
        width: 32,
        height: 41,
    }
}

fn assert_single_character_parsed_correctly(orientation: OrdinateOrientation, y: i32) {
    let char_positions = parse(UNDERSCORE_CHARACTER, orientation);
    assert_eq!(char_positions.len(), UNDERSCORE_CHARACTER.len());
    let char_position =
        create_char_position(page_rect_for_underscore(), screen_rect_for_underscore(y));
    assert_char_positions_equal(&char_positions[0], &char_position);
}

fn assert_text_parsed_correctly(orientation: OrdinateOrientation, line_count: u32, ys: [i32; 4]) {
    assert!(line_count != 0);
    let mut text = RUST_WORD.to_string();
    for _ in 1..line_count {
        text.push('\n');
        text.push_str(RUST_WORD);
    }
    let char_positions = parse(RUST_WORD, orientation);
    assert_eq!(char_positions.len(), RUST_WORD.len() * line_count as usize);
    const LINE_HEIGHT: i32 = 80;
    for line in 0..line_count {
        let line = line as i32;
        let page_rects = page_rects_for_rust_word();
        let screen_rects = screen_rects_for_rust_word([
            line * LINE_HEIGHT + ys[0],
            line * LINE_HEIGHT + ys[1],
            line * LINE_HEIGHT + ys[2],
            line * LINE_HEIGHT + ys[3],
        ]);
        let iter = page_rects
            .into_iter()
            .zip(screen_rects.into_iter())
            .enumerate();
        for (i, (page_rect, screen_rect)) in iter {
            let actual = &char_positions[line as usize * RUST_WORD.len() + i];
            let expected = create_char_position(page_rect, screen_rect);
            assert_char_positions_equal(actual, &expected);
        }
    }
}

fn assert_letters_with_kerning_parsed_correctly(orientation: OrdinateOrientation, ys: [i32; 3]) {
    let char_positions = parse(YOU_WORD, orientation);
    assert_eq!(char_positions.len(), YOU_WORD.len());
    let page_rects = page_rects_for_you_word();
    let screen_rects = vec![
        screen_rect_for_capital_y_in_you_word(ys[0]),
        screen_rect_for_o_in_you_word(ys[1]),
        screen_rect_for_u_in_you_word(ys[2]),
    ];
    let iter = page_rects
        .into_iter()
        .zip(screen_rects.into_iter())
        .enumerate();
    for (i, (page_rect, screen_rect)) in iter {
        let actual = &char_positions[i];
        let expected = create_char_position(page_rect, screen_rect);
        assert_char_positions_equal(actual, &expected);
    }
}

#[test]
fn pages_parsed_correctly() {
    let bmfont = create_bmfont(OrdinateOrientation::TopToBottom);
    assert_eq!(bmfont.pages().next(), Some("font.png"));
}

#[test]
fn single_character_for_bottom_to_top_orientation_parsed_correctly() {
    assert_single_character_parsed_correctly(OrdinateOrientation::BottomToTop, -16);
}

#[test]
fn single_character_for_top_to_bottom_orientation_parsed_correctly() {
    assert_single_character_parsed_correctly(OrdinateOrientation::TopToBottom, 66);
}

#[test]
fn multiple_lines_for_top_to_bottom_orientation_parsed_correctly() {
    assert_text_parsed_correctly(OrdinateOrientation::TopToBottom, 1, [5, 19, 18, 6]);
}

#[test]
fn multiple_lines_for_bottom_to_top_orientation_parsed_correctly() {
    assert_text_parsed_correctly(OrdinateOrientation::BottomToTop, 1, [-2, -3, -2, -3]);
}

#[test]
fn letters_with_kerning_for_top_to_bottom_orientation_parsed_correctly() {
    assert_letters_with_kerning_parsed_correctly(OrdinateOrientation::TopToBottom, [5, 18, 19]);
}

#[test]
fn letters_with_kerning_for_bottom_to_top_orientation_parsed_correctly() {
    assert_letters_with_kerning_parsed_correctly(OrdinateOrientation::BottomToTop, [-2, -2, -3]);
}

#[cfg(feature = "parse-error")]
#[test]
fn missing_character_handled_correctly() {
    let bmfont = create_bmfont(OrdinateOrientation::TopToBottom);
    match bmfont.parse("Ř") {
        Err(error) => assert_eq!(error.missing_characters, vec!['Ř']),
        Ok(_) => panic!(),
    }
}

#[cfg(not(feature = "parse-error"))]
#[test]
fn missing_character_handled_correctly() {
    let bmfont = create_bmfont(OrdinateOrientation::TopToBottom);
    assert_eq!(bmfont.parse("Ř").count(), 0);
}

#[cfg(feature = "parse-error")]
#[test]
fn unsupported_character_handled_correctly() {
    let bmfont = create_bmfont(OrdinateOrientation::TopToBottom);
    match bmfont.parse("𐃌") {
        Err(error) => assert_eq!(error.unsupported_characters, vec!['𐃌']),
        Ok(_) => panic!(),
    }
}

#[cfg(not(feature = "parse-error"))]
#[test]
fn unsupported_character_handled_correctly() {
    let bmfont = create_bmfont(OrdinateOrientation::TopToBottom);
    assert_eq!(bmfont.parse("𐃌").count(), 0);
}
