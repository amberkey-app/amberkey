//! Print-at-home kit PDF (spec/share-card.md). Generated entirely client-side;
//! mnemonics never leave the device. Per holder: card page, instruction sheet,
//! sealed circle-directory page (fold and tape).
//!
//! ponytail: printpdf 0.7 doesn't compile to wasm32, so this uses a ~150-line
//! built-in PDF writer below (text in the 14 standard fonts + rects + lines is
//! all a card needs). Output is deterministic. Swap for a real PDF crate if
//! layout needs ever exceed these primitives.

use crate::error::{Error, Result};
use qrcode::{EcLevel, QrCode};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct KitInput {
    pub case_number: String,
    pub group_threshold: u8,
    /// mnemonics per group, in group order
    pub groups: Vec<KitGroup>,
    pub tool_hash: String,
    pub minisign_fingerprint: String,
    /// Sealed-envelope content: who the other holders are, plain text lines.
    pub circle_directory_text: String,
    /// Plain-English recovery path sentences the owner confirmed in the wizard.
    pub recovery_paths_text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KitGroup {
    pub name: String,
    pub member_threshold: u8,
    pub mnemonics: Vec<String>,
    /// Holder display names, parallel to `mnemonics`. Printed ONLY on the
    /// instruction sheet ("give this card to …"), never on the card itself
    /// (spec/share-card.md: cards carry no names).
    #[serde(default)]
    pub member_names: Vec<String>,
}

// ---------- minimal PDF writer ----------

const PT_PER_MM: f32 = 72.0 / 25.4;
const PAGE_W: f32 = 210.0; // A4 mm
const PAGE_H: f32 = 297.0;
const MARGIN: f32 = 18.0;

#[derive(Clone, Copy)]
enum Font {
    Regular, // Helvetica       -> /F1
    Bold,    // Helvetica-Bold  -> /F2
    Mono,    // Courier         -> /F3
}

impl Font {
    fn res(self) -> &'static str {
        match self {
            Font::Regular => "F1",
            Font::Bold => "F2",
            Font::Mono => "F3",
        }
    }
}

#[derive(Default)]
struct Page {
    ops: String,
}

impl Page {
    /// x, y in mm from the top-left of the page.
    fn text(&mut self, font: Font, size: f32, x: f32, y: f32, s: &str) {
        let escaped: String = s
            .chars()
            .map(|c| match c {
                '(' => "\\(".to_string(),
                ')' => "\\)".to_string(),
                '\\' => "\\\\".to_string(),
                c if c.is_ascii() => c.to_string(),
                // WinAnsi middle dot for the interpuncts used in copy; drop the rest.
                '·' => "\\267".to_string(),
                '—' => "-".to_string(),
                '"' | '“' | '”' => "\"".to_string(),
                '’' | '‘' => "'".to_string(),
                _ => "?".to_string(),
            })
            .collect();
        self.ops.push_str(&format!(
            "BT /{} {:.1} Tf {:.2} {:.2} Td ({}) Tj ET\n",
            font.res(),
            size,
            x * PT_PER_MM,
            (PAGE_H - y) * PT_PER_MM,
            escaped
        ));
    }

    /// Filled rectangle; x, y in mm from the top-left, h extends downward.
    fn rect_fill(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.ops.push_str(&format!(
            "{:.2} {:.2} {:.2} {:.2} re f\n",
            x * PT_PER_MM,
            (PAGE_H - y - h) * PT_PER_MM,
            w * PT_PER_MM,
            h * PT_PER_MM
        ));
    }

    fn rect_stroke(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.ops.push_str(&format!(
            "{:.2} {:.2} {:.2} {:.2} re S\n",
            x * PT_PER_MM,
            (PAGE_H - y - h) * PT_PER_MM,
            w * PT_PER_MM,
            h * PT_PER_MM
        ));
    }

    fn hline(&mut self, y: f32, x0: f32, x1: f32) {
        self.ops.push_str(&format!(
            "{:.2} {:.2} m {:.2} {:.2} l S\n",
            x0 * PT_PER_MM,
            (PAGE_H - y) * PT_PER_MM,
            x1 * PT_PER_MM,
            (PAGE_H - y) * PT_PER_MM
        ));
    }
}

/// Assemble a PDF 1.4 file: catalog, page tree, 3 Type1 builtin fonts, one
/// uncompressed content stream per page, correct xref offsets.
fn assemble(pages: &[Page]) -> Vec<u8> {
    let n_pages = pages.len();
    // Object numbering: 1 catalog, 2 pages, 3-5 fonts, then per page i:
    // 6+2i page object, 7+2i content stream.
    let kids: Vec<String> = (0..n_pages).map(|i| format!("{} 0 R", 6 + 2 * i)).collect();
    let mut objects: Vec<String> = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), n_pages),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>".to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Courier /Encoding /WinAnsiEncoding >>".to_string(),
    ];
    let media = format!("[0 0 {:.2} {:.2}]", PAGE_W * PT_PER_MM, PAGE_H * PT_PER_MM);
    for (i, page) in pages.iter().enumerate() {
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox {media} \
             /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 5 0 R >> >> /Contents {} 0 R >>",
            7 + 2 * i
        ));
        objects.push(format!("<< /Length {} >>\nstream\n{}endstream", page.ops.len(), page.ops));
    }

    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for (i, obj) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", i + 1, obj).as_bytes());
    }
    let xref_at = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for off in offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref_at
        )
        .as_bytes(),
    );
    out
}

/// Render `data` as a self-contained SVG QR code (no external assets), for the
/// TOTP-enrollment view and anywhere a scannable code helps. Dark modules are
/// one merged path; the SVG is safe to inline (we generate every byte).
pub fn qr_svg(data: &str) -> Result<String> {
    let code = QrCode::with_error_correction_level(data, EcLevel::M)
        .map_err(|e| Error::Bundle(format!("qr: {e}")))?;
    let n = code.width();
    let quiet = 2; // quiet zone in modules
    let dim = n + quiet * 2;
    let mut path = String::new();
    for (i, c) in code.to_colors().iter().enumerate() {
        if *c == qrcode::Color::Dark {
            let x = (i % n) + quiet;
            let y = (i / n) + quiet;
            path.push_str(&format!("M{x} {y}h1v1h-1z"));
        }
    }
    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {dim} {dim}\" \
         shape-rendering=\"crispEdges\" role=\"img\" aria-label=\"QR code\">\
         <rect width=\"{dim}\" height=\"{dim}\" fill=\"#ffffff\"/>\
         <path d=\"{path}\" fill=\"#000000\"/></svg>"
    ))
}

// ---------- kit layout ----------

pub fn generate_kit_pdf(input: &KitInput) -> Result<Vec<u8>> {
    // Page 1 is an owner-only distribution guide mapping each card ID to its
    // intended holder. It never leaves the owner's hands, so holder names stay
    // off the cards and off the sheets those holders keep.
    let mut pages = vec![distribution_page(input)];
    for (gi, group) in input.groups.iter().enumerate() {
        for (mi, mnemonic) in group.mnemonics.iter().enumerate() {
            let card_id = crate::card::ShareCard::card_id(&input.case_number, gi as u8, mi as u8);
            pages.push(card_page(input, group, &card_id, mnemonic)?);
            pages.push(instruction_page(input, &card_id));
            pages.push(sealed_page(input));
        }
    }
    Ok(assemble(&pages))
}

/// Owner-only first page: which card goes to whom. Marked to keep or shred.
fn distribution_page(input: &KitInput) -> Page {
    let mut p = Page::default();
    p.text(Font::Bold, 16.0, MARGIN, 26.0, "For you only — do not hand this page out");
    p.hline(31.0, MARGIN, PAGE_W - MARGIN);
    for (i, line) in [
        "This first page is your distribution guide. Use it to hand the right card",
        "and instruction sheet to each person, then keep it somewhere private or",
        "shred it. Nothing that names a holder is printed on the cards or on the",
        "sheets they keep, so this page is the only place the two are linked.",
    ]
    .iter()
    .enumerate()
    {
        p.text(Font::Regular, 10.5, MARGIN, 40.0 + i as f32 * 6.0, line);
    }

    let mut y = 74.0;
    p.text(Font::Bold, 12.0, MARGIN, y, &format!("Case {} — who gets which card", input.case_number));
    y += 9.0;
    for (gi, group) in input.groups.iter().enumerate() {
        p.text(Font::Bold, 10.5, MARGIN, y, &format!("Group \"{}\" (needs {})", group.name, group.member_threshold));
        y += 6.5;
        let count = group.mnemonics.len();
        for mi in 0..count {
            let card_id = crate::card::ShareCard::card_id(&input.case_number, gi as u8, mi as u8);
            let holder = group.member_names.get(mi).map(String::as_str).unwrap_or("(unnamed)");
            p.text(Font::Mono, 10.0, MARGIN + 6.0, y, &card_id);
            p.text(Font::Regular, 10.0, MARGIN + 60.0, y, &format!("-> {holder}"));
            y += 6.0;
        }
        y += 3.0;
    }

    y += 4.0;
    p.hline(y, MARGIN, PAGE_W - MARGIN);
    p.text(
        Font::Regular,
        9.5,
        MARGIN,
        y + 7.0,
        &format!("Cards from {} group(s) are needed to recover. Case {}.", input.group_threshold, input.case_number),
    );
    p
}

/// QR as vector squares: crisp at any print size, no image codecs.
fn draw_qr(page: &mut Page, data: &str, x: f32, y: f32, size_mm: f32) -> Result<()> {
    let code = QrCode::with_error_correction_level(data, EcLevel::M)
        .map_err(|e| Error::Bundle(format!("qr: {e}")))?;
    let n = code.width();
    let module = size_mm / n as f32;
    for (i, c) in code.to_colors().iter().enumerate() {
        if *c == qrcode::Color::Dark {
            let mx = (i % n) as f32 * module;
            let my = (i / n) as f32 * module;
            page.rect_fill(x + mx, y + my, module + 0.02, module + 0.02);
        }
    }
    Ok(())
}

fn card_page(input: &KitInput, group: &KitGroup, card_id: &str, mnemonic: &str) -> Result<Page> {
    let mut p = Page::default();

    p.text(Font::Bold, 18.0, MARGIN, 28.0, "AmberKey Recovery Card");
    p.text(Font::Mono, 12.0, MARGIN, 38.0, &format!("Card {card_id}"));
    p.text(
        Font::Regular,
        10.0,
        MARGIN,
        45.0,
        &format!("Case {}  ·  group \"{}\"  ·  keep this card safe and private", input.case_number, group.name),
    );
    p.hline(50.0, MARGIN, PAGE_W - MARGIN);

    // Words in 4 columns
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    let rows = words.len().div_ceil(4);
    for (i, w) in words.iter().enumerate() {
        let col = i / rows;
        let row = i % rows;
        p.text(Font::Mono, 10.5, MARGIN + col as f32 * 44.0, 62.0 + row as f32 * 7.5, &format!("{:>2}. {w}", i + 1));
    }
    let words_bottom = 62.0 + rows as f32 * 7.5;

    draw_qr(&mut p, mnemonic, MARGIN, words_bottom + 4.0, 42.0)?;
    p.text(
        Font::Regular,
        9.0,
        MARGIN + 48.0,
        words_bottom + 14.0,
        "This QR holds the same words. Any standard SLIP-39 tool can read them.",
    );
    p.text(Font::Regular, 9.0, MARGIN + 48.0, words_bottom + 20.0, "Alone, this card unlocks nothing and names nobody.");

    let qr_bottom = words_bottom + 4.0 + 42.0;
    let y0 = qr_bottom + 10.0;
    p.hline(y0, MARGIN, PAGE_W - MARGIN);
    p.text(Font::Bold, 11.0, MARGIN, y0 + 8.0, "If you are asked to use this card:");
    p.text(Font::Regular, 10.0, MARGIN, y0 + 15.0, &format!("1. Go to {}  (works offline; save the page)", crate::RECOVERY_URL));
    p.text(Font::Regular, 10.0, MARGIN, y0 + 21.0, "2. Verify the tool before trusting it:");
    p.text(Font::Mono, 8.0, MARGIN + 6.0, y0 + 27.0, &format!("sha256: {}", input.tool_hash));
    p.text(Font::Mono, 8.0, MARGIN + 6.0, y0 + 32.0, &format!("minisign: {}", input.minisign_fingerprint));
    p.text(Font::Regular, 10.0, MARGIN, y0 + 39.0, "3. Follow the instruction sheet that came with this card.");

    // Cut-out border, sized to the content (word count varies by secret size).
    p.rect_stroke(MARGIN - 4.0, MARGIN, PAGE_W - 2.0 * MARGIN + 8.0, y0 + 45.0 - MARGIN);
    Ok(p)
}

fn instruction_page(input: &KitInput, card_id: &str) -> Page {
    let mut p = Page::default();
    p.text(Font::Bold, 16.0, MARGIN, 28.0, "If you are reading this, someone trusted you.");
    let para = [
        "The person who gave you this card asked you to help protect what they leave",
        "behind. Your card is one piece of a key. It does nothing alone, and you never",
        "need an account, an app, or a payment to use it.",
        "",
        "Nothing is expected of you now, except:",
        "  ·  Keep the card somewhere safe, like important papers.",
        "  ·  Once a quarter, you may get an email asking to confirm you still have it.",
        "  ·  If the worst happens, gather with the others and follow the steps below.",
    ];
    for (i, l) in para.iter().enumerate() {
        p.text(Font::Regular, 10.5, MARGIN, 40.0 + i as f32 * 6.0, l);
    }

    let y = 95.0;
    p.text(Font::Bold, 12.0, MARGIN, y, "When the time comes");
    let url_line = format!("     {}  — save the page (one file, works without internet)", crate::RECOVERY_URL);
    let steps = [
        "1. Get the recovery tool. In order of preference:",
        url_line.as_str(),
        "     github.com/amberkey-app/amberkey  (Releases)",
        "     codeberg.org/amberkey/amberkey  (mirror)",
        "     archive.org/details/amberkey  (Internet Archive)",
        "2. Verify it: the file's SHA-256 hash and signing key are printed on your card.",
        "3. Open the tool. It works offline; nothing you type leaves the computer.",
        "4. Enter the words from your card, and the words from the other cards as the",
        "   tool asks. It shows exactly how many are still needed.",
        "5. Load the bundle file (on the USB drive kept with the estate papers, or",
        "   downloaded from the link in the release email).",
        "6. The tool unlocks the instructions. Start with the first page: the executor",
        "   checklist. It says what to do, in order, including this in bold:",
        "   DO NOT cancel the phone number until every account is settled.",
    ];
    for (i, l) in steps.iter().enumerate() {
        p.text(Font::Regular, 10.0, MARGIN, y + 8.0 + i as f32 * 6.0, l);
    }

    let y2 = y + 8.0 + steps.len() as f32 * 6.0 + 8.0;
    p.text(Font::Bold, 12.0, MARGIN, y2, "Who can unlock, together");
    for (i, l) in input.recovery_paths_text.lines().take(6).enumerate() {
        p.text(Font::Regular, 10.0, MARGIN, y2 + 7.0 + i as f32 * 6.0, l);
    }
    p.text(
        Font::Regular,
        8.5,
        MARGIN,
        282.0,
        &format!("AmberKey instruction sheet · card {card_id} · case {} · amberkey.app", input.case_number),
    );
    p
}

fn sealed_page(input: &KitInput) -> Page {
    let mut p = Page::default();
    // Upper half stays on the OUTSIDE after folding, so it carries the label,
    // the case number (to tell sealed pages apart), and the fold direction.
    p.text(Font::Bold, 14.0, MARGIN, 28.0, "SEALED - do not open unless you are recovering");
    p.text(
        Font::Regular,
        10.0,
        MARGIN,
        36.0,
        &format!("Sealed circle directory · Case {} · lists the people who hold the other cards.", input.case_number),
    );
    p.text(
        Font::Regular,
        10.0,
        MARGIN,
        43.0,
        "To seal: fold the lower half BEHIND this one so this label stays on the outside, then tape the edges.",
    );
    p.hline(49.0, MARGIN, PAGE_W - MARGIN);

    // Fold line (mid-page). The lower half folds behind, hiding the directory
    // against the blank back of this half while this label shows on the front.
    p.hline(PAGE_H / 2.0, 5.0, PAGE_W - 5.0);
    p.text(Font::Regular, 8.0, MARGIN, PAGE_H / 2.0 - 2.0, "- fold here: lower half goes BEHIND -");

    // Directory content on the lower half (hidden when folded and taped)
    let mut y = PAGE_H / 2.0 + 14.0;
    p.text(Font::Bold, 12.0, MARGIN, y, "Your recovery circle");
    y += 8.0;
    for line in input.circle_directory_text.lines().take(24) {
        p.text(Font::Regular, 10.0, MARGIN, y, line);
        y += 6.0;
    }
    y += 4.0;
    p.text(
        Font::Regular,
        9.0,
        MARGIN,
        y,
        &format!("Cards from {} group(s) are needed. Case {}.", input.group_threshold, input.case_number),
    );
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> KitInput {
        KitInput {
            case_number: "AK7F3K".into(),
            group_threshold: 2,
            groups: vec![
                KitGroup { name: "Partner".into(), member_threshold: 1, mnemonics: vec!["academic acid acrobat romp change injury painting safari drug browser trash fridge busy finger standard angry similar overall prayer burden".into()], member_names: vec!["Sam Rivera".into()] },
                KitGroup { name: "Kids".into(), member_threshold: 2, mnemonics: vec![vec!["word"; 33].join(" "), vec!["beta"; 33].join(" ")], member_names: vec!["Alex".into(), "Bo".into()] },
            ],
            tool_hash: "73627b70af3d579aaa1175f88ab1484efc2cc20ab20c092d79f4cee357a73f20".into(),
            minisign_fingerprint: "RWTest123".into(),
            circle_directory_text: "Partner: Sam (sam@example.com)\nKids: A, B".into(),
            recovery_paths_text: "Sam and both kids can recover.\nThe kids alone cannot.".into(),
        }
    }

    #[test]
    fn generates_a_valid_pdf_for_the_default_template() {
        let pdf = generate_kit_pdf(&input()).unwrap();
        assert!(pdf.starts_with(b"%PDF-"), "not a pdf");
        // 1 owner distribution page + 3 holders x 3 pages
        let pages = pdf.windows(12).filter(|w| w == b"/Type /Page ").count();
        assert_eq!(pages, 10, "expected 10 page objects");
        let s = String::from_utf8_lossy(&pdf);
        assert!(s.contains("AK7F3K-G1-M1") && s.contains("AK7F3K-G2-M2"));
        assert!(s.contains("SEALED - do not open") && s.contains("goes BEHIND"));
        // Holder names live only on the owner-only distribution page, never on
        // the cards or the sheets holders keep.
        assert!(s.contains("do not hand this page out"));
        assert!(s.contains("Sam Rivera"));
    }

    #[test]
    fn qr_svg_is_valid() {
        let svg = qr_svg("otpauth://totp/AmberKey:a@b.co?secret=ABC&issuer=AmberKey").unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("viewBox") && svg.contains("<path"));
        assert!(qr_svg("").is_ok());
    }

    #[test]
    fn kit_pdf_is_deterministic() {
        assert_eq!(generate_kit_pdf(&input()).unwrap(), generate_kit_pdf(&input()).unwrap());
    }
}
