//! amberkey-recover: offline recovery from printed share cards.
//! Zero network access, zero external runtime dependencies. std-only arg parsing
//! to keep the trust anchor's dependency graph minimal.

use amberkey_core::{bundle, slip39};
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
amberkey-recover — reconstruct an AmberKey vault offline

USAGE:
    amberkey-recover --bundle <bundle.age> --out <dir> [--shares <file>]
    amberkey-recover --check-shares [--shares <file>]

OPTIONS:
    --bundle <file>   Encrypted continuity bundle (from USB, download, or backup)
    --out <dir>       Directory to write the decrypted packet into
    --shares <file>   File with one share mnemonic per line (otherwise typed interactively)
    --check-shares    Only validate shares and report quorum progress; decrypts nothing

Shares are the word phrases printed on the AmberKey cards. Type each card's
words on one line. This tool never touches the network.";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut bundle_path: Option<PathBuf> = None;
    let mut out_dir: Option<PathBuf> = None;
    let mut shares_file: Option<PathBuf> = None;
    let mut check_only = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bundle" => bundle_path = Some(next_value(&mut args, "--bundle")?),
            "--out" => out_dir = Some(next_value(&mut args, "--out")?),
            "--shares" => shares_file = Some(next_value(&mut args, "--shares")?),
            "--check-shares" => check_only = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            other => return Err(format!("unknown argument {other:?}\n\n{USAGE}")),
        }
    }

    let mnemonics = read_shares(shares_file.as_deref())?;
    if mnemonics.is_empty() {
        return Err("no shares entered".into());
    }

    if check_only {
        return check_shares(&mnemonics);
    }

    let bundle_path = bundle_path.ok_or(format!("--bundle is required\n\n{USAGE}"))?;
    let out_dir = out_dir.ok_or(format!("--out is required\n\n{USAGE}"))?;

    let scalar = slip39::combine_mnemonics(&mnemonics, b"")
        .map_err(|e| format!("could not reconstruct the vault key: {e}"))?;
    let identity = bundle::identity_from_bytes(&scalar).map_err(|e| e.to_string())?;
    println!("vault key reconstructed.");

    let ciphertext = std::fs::read(&bundle_path)
        .map_err(|e| format!("cannot read {}: {e}", bundle_path.display()))?;
    let files = bundle::decrypt_bundle(&identity, &ciphertext)
        .map_err(|e| format!("could not decrypt the bundle: {e}"))?;
    let manifest = bundle::read_manifest(&files).map_err(|e| e.to_string())?;
    println!(
        "bundle decrypted: {} (created {}, schema v{})",
        manifest.owner_name, manifest.created_at, manifest.schema_version
    );

    for f in &files {
        let rel = sanitize(&f.path).ok_or(format!("bundle contains unsafe path {:?}", f.path))?;
        let dest = out_dir.join(rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&dest, &f.data).map_err(|e| e.to_string())?;
    }
    println!("{} files written to {}", files.len(), out_dir.display());
    println!();
    println!("START HERE: open packet/executor-checklist.md in the output directory.");
    println!("Do not cancel the deceased's phone number until every account is settled.");
    Ok(())
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next().map(PathBuf::from).ok_or(format!("{flag} needs a value"))
}

/// Reject absolute paths and any `..` traversal when extracting.
fn sanitize(path: &str) -> Option<PathBuf> {
    let p = Path::new(path);
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => return None,
        }
    }
    if out.as_os_str().is_empty() {
        None
    } else {
        Some(out)
    }
}

fn read_shares(file: Option<&Path>) -> Result<Vec<String>, String> {
    let lines: Vec<String> = match file {
        Some(f) => std::fs::read_to_string(f)
            .map_err(|e| format!("cannot read {}: {e}", f.display()))?
            .lines()
            .map(str::to_string)
            .collect(),
        None => {
            let stdin = std::io::stdin();
            if stdin.is_terminal() {
                println!("Enter each card's words on one line. Finish with an empty line.");
            }
            let mut lines = Vec::new();
            for line in stdin.lock().lines() {
                let line = line.map_err(|e| e.to_string())?;
                if line.trim().is_empty() {
                    break;
                }
                lines.push(line);
                let _ = std::io::stdout().flush();
            }
            lines
        }
    };
    Ok(lines
        .into_iter()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

fn check_shares(mnemonics: &[String]) -> Result<(), String> {
    let mut ok = true;
    for (i, m) in mnemonics.iter().enumerate() {
        match slip39::Share::from_mnemonic(m) {
            Ok(s) => println!(
                "share {}: valid — group {} of {}, member {}, needs {} from this group, {} group(s) total",
                i + 1,
                s.group_index + 1,
                s.group_count,
                s.member_index + 1,
                s.member_threshold,
                s.group_threshold
            ),
            Err(e) => {
                ok = false;
                println!("share {}: INVALID — {e}", i + 1);
            }
        }
    }
    match slip39::combine_mnemonics(mnemonics, b"") {
        Ok(_) => println!("quorum COMPLETE: these shares reconstruct the vault key."),
        Err(e) => println!("quorum not yet complete: {e}"),
    }
    if ok {
        Ok(())
    } else {
        Err("some shares were invalid".into())
    }
}
