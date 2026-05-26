//! Verify an OpenWebHMI audit-log hash chain.

use std::path::PathBuf;

use openwebhmi_audit_log::AuditLog;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let Some(path) = args.next().map(PathBuf::from) else {
        eprintln!("usage: audit-verify <path-to-audit-db>");
        std::process::exit(2);
    };
    if args.next().is_some() {
        eprintln!("usage: audit-verify <path-to-audit-db>");
        std::process::exit(2);
    }

    let log = match AuditLog::open(path, 0) {
        Ok(log) => log,
        Err(err) => {
            eprintln!("ERROR: failed to open audit log: {err}");
            std::process::exit(2);
        }
    };
    match log.verify_chain() {
        Ok(count) => {
            println!("OK: {count} entries verified");
        }
        Err(err) => {
            println!(
                "BROKEN at id {}: expected {}, found {}",
                err.first_bad_id, err.expected_hash, err.found_hash
            );
            std::process::exit(1);
        }
    }
}
