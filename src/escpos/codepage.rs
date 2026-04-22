//! Codepage decoding for ESC/POS text bytes.
//!
//! ESC/POS printers do not transmit UTF-8: each byte in the text stream
//! refers to a glyph in the currently-selected codepage (set via `ESC t n`).
//! This module maps bytes 0x80..=0xFF to their Unicode equivalent for the
//! Latin codepages most commonly used in point-of-sale receipts.
//!
//! Bytes 0x00..=0x7F are always ASCII and never use the table.

/// Replacement character used for positions that are undefined in a codepage.
const UNDEF: char = '\u{FFFD}';

/// CP437 — original IBM PC / "USA, Standard Europe". ESC t n=0.
#[rustfmt::skip]
const CP437: [char; 128] = [
    'Ç','ü','é','â','ä','à','å','ç','ê','ë','è','ï','î','ì','Ä','Å',
    'É','æ','Æ','ô','ö','ò','û','ù','ÿ','Ö','Ü','¢','£','¥','₧','ƒ',
    'á','í','ó','ú','ñ','Ñ','ª','º','¿','⌐','¬','½','¼','¡','«','»',
    '░','▒','▓','│','┤','╡','╢','╖','╕','╣','║','╗','╝','╜','╛','┐',
    '└','┴','┬','├','─','┼','╞','╟','╚','╔','╩','╦','╠','═','╬','¤',
    '╨','╤','╥','╙','╘','╒','╓','╫','╪','┘','┌','█','▄','▌','▐','▀',
    'α','ß','Γ','π','Σ','σ','µ','τ','Φ','Θ','Ω','δ','∞','φ','ε','∩',
    '≡','±','≥','≤','⌠','⌡','÷','≈','°','∙','·','√','ⁿ','²','■','\u{00A0}',
];

/// CP850 — Multilingual Latin 1. ESC t n=2.
#[rustfmt::skip]
const CP850: [char; 128] = [
    'Ç','ü','é','â','ä','à','å','ç','ê','ë','è','ï','î','ì','Ä','Å',
    'É','æ','Æ','ô','ö','ò','û','ù','ÿ','Ö','Ü','ø','£','Ø','×','ƒ',
    'á','í','ó','ú','ñ','Ñ','ª','º','¿','®','¬','½','¼','¡','«','»',
    '░','▒','▓','│','┤','Á','Â','À','©','╣','║','╗','╝','¢','¥','┐',
    '└','┴','┬','├','─','┼','ã','Ã','╚','╔','╩','╦','╠','═','╬','¤',
    'ð','Ð','Ê','Ë','È','ı','Í','Î','Ï','┘','┌','█','▄','¦','Ì','▀',
    'Ó','ß','Ô','Ò','õ','Õ','µ','þ','Þ','Ú','Û','Ù','ý','Ý','¯','´',
    '\u{00AD}','±','‗','¾','¶','§','÷','¸','°','¨','·','¹','³','²','■','\u{00A0}',
];

/// CP858 — CP850 with € replacing `ı` at 0xD5. ESC t n=19.
const fn build_cp858() -> [char; 128] {
    let mut table = CP850;
    table[0xD5 - 0x80] = '€';
    table
}
const CP858: [char; 128] = build_cp858();

/// Windows-1252 (WPC1252) — Latin 1 Windows. ESC t n=16.
#[rustfmt::skip]
const CP1252: [char; 128] = [
    '€', UNDEF,'‚','ƒ','„','…','†','‡','ˆ','‰','Š','‹','Œ', UNDEF,'Ž', UNDEF,
    UNDEF,'\u{2018}','\u{2019}','\u{201C}','\u{201D}','•','–','—','˜','™','š','›','œ', UNDEF,'ž','Ÿ',
    '\u{00A0}','¡','¢','£','¤','¥','¦','§','¨','©','ª','«','¬','\u{00AD}','®','¯',
    '°','±','²','³','´','µ','¶','·','¸','¹','º','»','¼','½','¾','¿',
    'À','Á','Â','Ã','Ä','Å','Æ','Ç','È','É','Ê','Ë','Ì','Í','Î','Ï',
    'Ð','Ñ','Ò','Ó','Ô','Õ','Ö','×','Ø','Ù','Ú','Û','Ü','Ý','Þ','ß',
    'à','á','â','ã','ä','å','æ','ç','è','é','ê','ë','ì','í','î','ï',
    'ð','ñ','ò','ó','ô','õ','ö','÷','ø','ù','ú','û','ü','ý','þ','ÿ',
];

/// ISO 8859-15 (Latin-9) — ISO 8859-1 with €, Š/š, Ž/ž, Œ/œ, Ÿ. ESC t n=40.
#[rustfmt::skip]
const ISO8859_15: [char; 128] = [
    UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,
    UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,UNDEF,
    '\u{00A0}','¡','¢','£','€','¥','Š','§','š','©','ª','«','¬','\u{00AD}','®','¯',
    '°','±','²','³','Ž','µ','¶','·','ž','¹','º','»','Œ','œ','Ÿ','¿',
    'À','Á','Â','Ã','Ä','Å','Æ','Ç','È','É','Ê','Ë','Ì','Í','Î','Ï',
    'Ð','Ñ','Ò','Ó','Ô','Õ','Ö','×','Ø','Ù','Ú','Û','Ü','Ý','Þ','ß',
    'à','á','â','ã','ä','å','æ','ç','è','é','ê','ë','ì','í','î','ï',
    'ð','ñ','ò','ó','ô','õ','ö','÷','ø','ù','ú','û','ü','ý','þ','ÿ',
];

fn table_for(codepage: u8) -> &'static [char; 128] {
    match codepage {
        0 => &CP437,
        2 => &CP850,
        16 => &CP1252,
        19 => &CP858,
        40 => &ISO8859_15,
        _ => &CP437,
    }
}

/// Decode a byte stream according to the given ESC/POS codepage.
pub fn decode(codepage: u8, bytes: &[u8]) -> String {
    let table = table_for(codepage);
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        if b < 0x80 {
            out.push(b as char);
        } else {
            out.push(table[(b - 0x80) as usize]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_passthrough() {
        assert_eq!(decode(0, b"Hello 123"), "Hello 123");
        assert_eq!(decode(16, b"Hello 123"), "Hello 123");
    }

    #[test]
    fn cp437_accented_latin() {
        // ñ=0xA4, á=0xA0, ¿=0xA8, ¡=0xAD
        assert_eq!(decode(0, &[0xA4]), "ñ");
        assert_eq!(decode(0, &[0xA0, 0xA1, 0xA2, 0xA3]), "áíóú");
        assert_eq!(decode(0, &[0xA8, b'C', b'o', b'm', b'o', b'?']), "¿Como?");
    }

    #[test]
    fn cp850_latin() {
        // Á=0xB5, ç=0x87, õ=0xE4
        assert_eq!(decode(2, &[0xB5, 0x87, 0xE4]), "Áçõ");
    }

    #[test]
    fn cp858_euro() {
        // 0xD5 should be € in CP858 vs ı in CP850
        assert_eq!(decode(19, &[0xD5]), "€");
        assert_eq!(decode(2, &[0xD5]), "ı");
    }

    #[test]
    fn cp1252_quotes_and_euro() {
        assert_eq!(decode(16, &[0x80]), "€");
        assert_eq!(decode(16, &[0x91, 0x92]), "\u{2018}\u{2019}");
        assert_eq!(decode(16, &[0xF1]), "ñ");
    }

    #[test]
    fn iso8859_15_euro() {
        assert_eq!(decode(40, &[0xA4]), "€");
        assert_eq!(decode(40, &[0xF1]), "ñ");
    }

    #[test]
    fn unknown_codepage_falls_back_to_cp437() {
        assert_eq!(decode(99, &[0xA4]), "ñ");
    }
}
