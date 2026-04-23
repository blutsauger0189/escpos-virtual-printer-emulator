use crate::escpos::codepage;
use crate::escpos::commands::{EscPosCommand, Font, Justification};
use crate::escpos::qr;
use anyhow::Result;

pub struct EscPosParser {
    buffer: Vec<u8>,
    codepage: u8,
    qr_module_size: u8,
    qr_ec_byte: u8,
    qr_data: Vec<u8>,
}

impl EscPosParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            codepage: 0,
            qr_module_size: 3,
            qr_ec_byte: 49,
            qr_data: Vec::new(),
        }
    }

    pub fn parse_stream(&mut self, data: &[u8]) -> Result<Vec<EscPosCommand>> {
        self.buffer.extend_from_slice(data);
        let mut commands = Vec::new();
        let mut i = 0;

        while i < self.buffer.len() {
            match self.buffer[i] {
                b'\n' => {
                    commands.push(EscPosCommand::NewLine);
                    i += 1;
                }
                b'\r' => {
                    commands.push(EscPosCommand::CarriageReturn);
                    i += 1;
                }
                0x1B => {
                    // ESC sequence
                    if i + 1 >= self.buffer.len() {
                        break; // Wait for more data
                    }
                    match self.parse_esc_command(&self.buffer[i..]) {
                        Ok(Some((cmd, consumed))) => {
                            if let EscPosCommand::SetCodepage(cp) = cmd {
                                self.codepage = cp;
                            }
                            commands.push(cmd);
                            i += consumed;
                        }
                        Ok(None) => break, // Incomplete, wait for more
                        Err(_) => { i += 2; } // Skip bad ESC sequence
                    }
                }
                0x1D => {
                    // GS sequence
                    if i + 1 >= self.buffer.len() {
                        break;
                    }
                    let chunk = self.buffer[i..].to_vec();
                    match self.parse_gs_command(&chunk) {
                        Ok(Some((maybe_cmd, consumed))) => {
                            if let Some(cmd) = maybe_cmd {
                                commands.push(cmd);
                            }
                            i += consumed;
                        }
                        Ok(None) => break,
                        Err(_) => { i += 2; }
                    }
                }
                _ => {
                    // Normal text bytes
                    let text_start = i;
                    while i < self.buffer.len()
                        && self.buffer[i] != 0x1B
                        && self.buffer[i] != 0x1D
                        && self.buffer[i] != b'\n'
                        && self.buffer[i] != b'\r'
                    {
                        i += 1;
                    }
                    if i > text_start {
                        let text = codepage::decode(self.codepage, &self.buffer[text_start..i]);
                        if !text.is_empty() {
                            commands.push(EscPosCommand::Text(text));
                        }
                    }
                }
            }
        }

        if i > 0 {
            self.buffer.drain(0..i);
        }

        Ok(commands)
    }

    /// Parse ESC (0x1B) commands. Returns (command, bytes_consumed).
    fn parse_esc_command(&self, data: &[u8]) -> Result<Option<(EscPosCommand, usize)>> {
        if data.len() < 2 {
            return Ok(None);
        }

        match data[1] {
            // Initialize printer
            b'@' => Ok(Some((EscPosCommand::InitializePrinter, 2))),

            // Select font
            b'M' => {
                if data.len() < 3 { return Ok(None); }
                let font = match data[2] {
                    0 => Font::FontA,
                    1 => Font::FontB,
                    2 => Font::FontC,
                    _ => Font::FontA,
                };
                Ok(Some((EscPosCommand::SetFont(font), 3)))
            }

            // Justification
            b'a' => {
                if data.len() < 3 { return Ok(None); }
                let j = match data[2] {
                    0 => Justification::Left,
                    1 => Justification::Center,
                    2 => Justification::Right,
                    _ => Justification::Left,
                };
                Ok(Some((EscPosCommand::SetJustification(j), 3)))
            }

            // Emphasis on/off
            b'E' => Ok(Some((EscPosCommand::SetEmphasis(true), 2))),
            b'F' => Ok(Some((EscPosCommand::SetEmphasis(false), 2))),

            // Underline
            b'-' => {
                if data.len() < 3 { return Ok(None); }
                Ok(Some((EscPosCommand::SetUnderline(data[2] != 0), 3)))
            }

            // Italic on/off
            b'4' => Ok(Some((EscPosCommand::SetItalic(true), 2))),
            b'5' => Ok(Some((EscPosCommand::SetItalic(false), 2))),

            // Line height
            b'3' => {
                if data.len() < 3 { return Ok(None); }
                Ok(Some((EscPosCommand::SetLineHeight(data[2] as u32), 3)))
            }

            // Font size / print mode
            b'!' => {
                if data.len() < 3 { return Ok(None); }
                Ok(Some((EscPosCommand::SetFontSize(data[2] as u32), 3)))
            }

            // Codepage selection (ESC t n)
            b't' => {
                if data.len() < 3 { return Ok(None); }
                Ok(Some((EscPosCommand::SetCodepage(data[2]), 3)))
            }

            // Cut paper
            b'm' | b'i' => Ok(Some((EscPosCommand::CutPaper, 2))),

            // Paper feed
            b'J' => {
                if data.len() < 3 { return Ok(None); }
                Ok(Some((EscPosCommand::LineFeed, 3)))
            }

            // Bit image (ESC *) — simplified
            b'*' => {
                if data.len() < 4 { return Ok(None); }
                let m = data[2];
                let nl = data[3] as u16;
                if data.len() < 5 { return Ok(None); }
                let nh = data[4] as u16;
                let n_dots = nl + nh * 256;
                let bytes_per_col: u16 = match m { 0 | 1 => 1, 32 | 33 => 3, _ => 1 };
                let total = bytes_per_col as usize * n_dots as usize;
                let consumed = 5 + total;
                if data.len() < consumed { return Ok(None); }
                let image_data = data[5..consumed].to_vec();
                Ok(Some((EscPosCommand::PrintImage(image_data), consumed)))
            }

            _ => {
                Ok(Some((EscPosCommand::Unknown(data[..2].to_vec()), 2)))
            }
        }
    }

    /// Parse GS (0x1D) commands. Returns (optional command, bytes_consumed).
    /// Some subcommands (e.g. QR state setters) only mutate parser state and
    /// do not emit a command — those return `(None, consumed)`.
    fn parse_gs_command(
        &mut self,
        data: &[u8],
    ) -> Result<Option<(Option<EscPosCommand>, usize)>> {
        if data.len() < 2 {
            return Ok(None);
        }

        match data[1] {
            // GS v 0 — Print raster bit image
            b'v' => {
                if data.len() < 8 { return Ok(None); }
                // GS v 0 m xL xH yL yH d1...dk
                let _mode = data[3]; // 0=normal, 1=double-width, 2=double-height, 3=both
                let x_l = data[4] as u16;
                let x_h = data[5] as u16;
                let y_l = data[6] as u16;
                let y_h = data[7] as u16;
                let width_bytes = x_l + x_h * 256; // bytes per row
                let height = y_l + y_h * 256;       // number of rows
                let total = width_bytes as usize * height as usize;
                let consumed = 8 + total;
                if data.len() < consumed { return Ok(None); }
                let image_data = data[8..consumed].to_vec();
                Ok(Some((
                    Some(EscPosCommand::PrintRasterImage { width_bytes, height, data: image_data }),
                    consumed,
                )))
            }

            // GS V — Cut paper (with variants)
            b'V' => {
                if data.len() < 3 { return Ok(None); }
                match data[2] {
                    0 | 1 => Ok(Some((Some(EscPosCommand::CutPaper), 3))),
                    65 | 66 => {
                        // GS V 65/66 n — need one more byte
                        if data.len() < 4 { return Ok(None); }
                        Ok(Some((Some(EscPosCommand::CutPaper), 4)))
                    }
                    _ => Ok(Some((Some(EscPosCommand::CutPaper), 3))),
                }
            }

            // GS ( k — 2D symbol / QR code
            b'(' => self.parse_gs_paren(data),

            _ => {
                Ok(Some((Some(EscPosCommand::Unknown(data[..2].to_vec())), 2)))
            }
        }
    }

    /// Parse `GS ( k pL pH cn fn [parameters...]`.
    ///
    /// Total command length is `5 + (pL + pH*256)` bytes. Only `cn = 0x31`
    /// (QR code) is handled; other symbologies are consumed (so the byte
    /// stream stays aligned) and reported as `Unknown`.
    fn parse_gs_paren(
        &mut self,
        data: &[u8],
    ) -> Result<Option<(Option<EscPosCommand>, usize)>> {
        if data.len() < 5 { return Ok(None); }
        if data[2] != b'k' {
            // GS ( <other> — not 2D symbol. Skip the two bytes we recognized.
            return Ok(Some((Some(EscPosCommand::Unknown(data[..3].to_vec())), 3)));
        }
        let p_l = data[3] as usize;
        let p_h = data[4] as usize;
        let total_len = 5 + p_l + p_h * 256;
        if data.len() < total_len { return Ok(None); }
        if total_len < 7 {
            // Malformed: not even room for cn + fn
            return Ok(Some((Some(EscPosCommand::Unknown(data[..total_len].to_vec())), total_len)));
        }

        let cn = data[5];
        let fn_code = data[6];
        let params = &data[7..total_len];

        if cn != 0x31 {
            // Not QR (e.g. PDF417 cn=0x30, AZTEC cn=0x35). Consume and skip.
            return Ok(Some((Some(EscPosCommand::Unknown(data[..total_len].to_vec())), total_len)));
        }

        match fn_code {
            // fn=65 — select model (params: n1 n2). Currently just consumed.
            65 => Ok(Some((None, total_len))),

            // fn=67 — set module size (params: n, 1..=16)
            67 => {
                if let Some(&n) = params.first() {
                    self.qr_module_size = n.clamp(1, 16);
                }
                Ok(Some((None, total_len)))
            }

            // fn=69 — set error-correction level (params: n, 48..=51 ⇒ L/M/Q/H)
            69 => {
                if let Some(&n) = params.first() {
                    self.qr_ec_byte = n;
                }
                Ok(Some((None, total_len)))
            }

            // fn=80 — store data in symbol storage area (params: m d1..dk, m=48)
            80 => {
                if !params.is_empty() {
                    self.qr_data = params[1..].to_vec();
                }
                Ok(Some((None, total_len)))
            }

            // fn=81 — print the stored symbol (params: m, m=48). Render now.
            81 => {
                let cmd = if self.qr_data.is_empty() {
                    None
                } else {
                    let ec_level = qr::ec_level_from_byte(self.qr_ec_byte);
                    qr::render_qr(&self.qr_data, self.qr_module_size, ec_level).map(
                        |(width_bytes, height, data)| EscPosCommand::PrintRasterImage {
                            width_bytes,
                            height,
                            data,
                        },
                    )
                };
                self.qr_data.clear();
                Ok(Some((cmd, total_len)))
            }

            // fn=82 — transmit size info (host→printer query); consume silently.
            _ => Ok(Some((None, total_len))),
        }
    }
}

impl Default for EscPosParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EscPosParser {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            codepage: self.codepage,
            qr_module_size: self.qr_module_size,
            qr_ec_byte: self.qr_ec_byte,
            qr_data: self.qr_data.clone(),
        }
    }
}
