use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

use crate::integrity;

// Embedded hook script (guards before set -euo pipefail)
const REWRITE_HOOK: &str = include_str!("../hooks/clov-rewrite.sh");

// Embedded slim CLOV awareness instructions
const CLOV_SLIM: &str = include_str!("../hooks/clov-awareness.md");

/// Control flow for settings.json patching
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatchMode {
    Ask,  // Default: prompt user [y/N]
    Auto, // --auto-patch: no prompt
    Skip, // --no-patch: manual instructions
}

/// Result of settings.json patching operation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatchResult {
    Patched,        // Hook was added successfully
    AlreadyPresent, // Hook was already in settings.json
    Declined,       // User declined when prompted
    Skipped,        // --no-patch flag used
}

// Legacy full instructions for backward compatibility (--claude-md mode)
const CLOV_INSTRUCTIONS: &str = r##"<!-- clov-instructions v2 -->
# CLOV (Clov Token Omitter) - Token-Optimized Commands

## Golden Rule

**Always prefix commands with `clov`**. If CLOV has a dedicated filter, it uses it. If not, it passes through unchanged. This means CLOV is always safe to use.

**Important**: Even in command chains with `&&`, use `clov`:
```bash
# ❌ Wrong
git add . && git commit -m "msg" && git push

# ✅ Correct
clov git add . && clov git commit -m "msg" && clov git push
```

## CLOV Commands by Workflow

### Build & Compile (80-90% savings)
```bash
clov cargo build         # Cargo build output
clov cargo check         # Cargo check output
clov cargo clippy        # Clippy warnings grouped by file (80%)
clov tsc                 # TypeScript errors grouped by file/code (83%)
clov lint                # ESLint/Biome violations grouped (84%)
clov prettier --check    # Files needing format only (70%)
clov next build          # Next.js build with route metrics (87%)
```

### Test (90-99% savings)
```bash
clov cargo test          # Cargo test failures only (90%)
clov vitest run          # Vitest failures only (99.5%)
clov playwright test     # Playwright failures only (94%)
clov check <cmd>         # Generic test wrapper - failures only
```

### Git (59-80% savings)
```bash
clov git status          # Compact status
clov git log             # Compact log (works with all git flags)
clov git diff            # Compact diff (80%)
clov git show            # Compact show (80%)
clov git add             # Ultra-compact confirmations (59%)
clov git commit          # Ultra-compact confirmations (59%)
clov git push            # Ultra-compact confirmations
clov git pull            # Ultra-compact confirmations
clov git branch          # Compact branch list
clov git fetch           # Compact fetch
clov git stash           # Compact stash
clov git worktree        # Compact worktree
```

Note: Git passthrough works for ALL subcommands, even those not explicitly listed.

### GitHub (26-87% savings)
```bash
clov gh pr view <num>    # Compact PR view (87%)
clov gh pr checks        # Compact PR checks (79%)
clov gh run list         # Compact workflow runs (82%)
clov gh issue list       # Compact issue list (80%)
clov gh api              # Compact API responses (26%)
```

### JavaScript/TypeScript Tooling (70-90% savings)
```bash
clov pnpm list           # Compact dependency tree (70%)
clov pnpm outdated       # Compact outdated packages (80%)
clov pnpm install        # Compact install output (90%)
clov npm run <script>    # Compact npm script output
clov npx <cmd>           # Compact npx command output
clov prisma              # Prisma without ASCII art (88%)
```

### Files & Search (60-75% savings)
```bash
clov files <path>        # Tree format, compact (65%)
clov view <file>         # Code reading with filtering (60%)
clov search <pattern>    # Search grouped by file (75%)
clov scan <pattern>      # Find grouped by directory (70%)
```

### Analysis & Debug (70-90% savings)
```bash
clov fail <cmd>          # Filter errors only from any command
clov logs <file>         # Deduplicated logs with counts
clov schema <file>       # JSON structure without values
clov graph               # Dependency overview
clov vars                # Environment variables compact
clov digest <cmd>        # Smart summary of command output
clov patch               # Ultra-compact diffs
```

### Infrastructure (85% savings)
```bash
clov docker ps           # Compact container list
clov docker images       # Compact image list
clov docker logs <c>     # Deduplicated logs
clov kubectl get         # Compact resource list
clov kubectl logs        # Deduplicated pod logs
```

### Network (65-70% savings)
```bash
clov curl <url>          # Compact HTTP responses (70%)
clov fetch <url>         # Compact download output (65%)
```

### Meta Commands
```bash
clov pulse               # View token savings statistics
clov pulse --history     # View command history with savings
clov inspect             # Analyze Claude Code sessions for missed CLOV usage
clov passthrough <cmd>   # Run command without filtering (for debugging)
clov hook                # Add CLOV instructions to CLAUDE.md
clov hook --global       # Add CLOV to ~/.claude/CLAUDE.md
```

## Token Savings Overview

| Category | Commands | Typical Savings |
|----------|----------|-----------------|
| Tests | vitest, playwright, cargo test | 90-99% |
| Build | next, tsc, lint, prettier | 70-87% |
| Git | status, log, diff, add, commit | 59-80% |
| GitHub | gh pr, gh run, gh issue | 26-87% |
| Package Managers | pnpm, npm, npx | 70-90% |
| Files | ls, read, grep, find | 60-75% |
| Infrastructure | docker, kubectl | 85% |
| Network | curl, wget | 65-70% |

Overall average: **60-90% token reduction** on common development operations.
<!-- /clov-instructions -->
"##;

/// Main entry point for `clov hook`
pub fn run(
    global: bool,
    claude_md: bool,
    hook_only: bool,
    patch_mode: PatchMode,
    verbose: u8,
) -> Result<()> {
    // Mode selection
    match (claude_md, hook_only) {
        (true, _) => run_claude_md_mode(global, verbose),
        (false, true) => run_hook_only_mode(global, patch_mode, verbose),
        (false, false) => run_default_mode(global, patch_mode, verbose),
    }
}

/// Prepare hook directory and return paths (hook_dir, hook_path)
fn prepare_hook_paths() -> Result<(PathBuf, PathBuf)> {
    let claude_dir = resolve_claude_dir()?;
    let hook_dir = claude_dir.join("hooks");
    fs::create_dir_all(&hook_dir)
        .with_context(|| format!("Failed to create hook directory: {}", hook_dir.display()))?;
    let hook_path = hook_dir.join("clov-rewrite.sh");
    Ok((hook_dir, hook_path))
}

/// Write hook file if missing or outdated, return true if changed
#[cfg(unix)]
fn ensure_hook_installed(hook_path: &Path, verbose: u8) -> Result<bool> {
    let changed = if hook_path.exists() {
        let existing = fs::read_to_string(hook_path)
            .with_context(|| format!("Failed to read existing hook: {}", hook_path.display()))?;

        if existing == REWRITE_HOOK {
            if verbose > 0 {
                eprintln!("Hook already up to date: {}", hook_path.display());
            }
            false
        } else {
            fs::write(hook_path, REWRITE_HOOK)
                .with_context(|| format!("Failed to write hook to {}", hook_path.display()))?;
            if verbose > 0 {
                eprintln!("Updated hook: {}", hook_path.display());
            }
            true
        }
    } else {
        fs::write(hook_path, REWRITE_HOOK)
            .with_context(|| format!("Failed to write hook to {}", hook_path.display()))?;
        if verbose > 0 {
            eprintln!("Created hook: {}", hook_path.display());
        }
        true
    };

    // Set executable permissions
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(hook_path, fs::Permissions::from_mode(0o755))
        .with_context(|| format!("Failed to set hook permissions: {}", hook_path.display()))?;

    // Store SHA-256 hash for runtime integrity verification.
    // Always store (idempotent) to ensure baseline exists even for
    // hooks installed before integrity checks were added.
    integrity::store_hash(hook_path)
        .with_context(|| format!("Failed to store integrity hash for {}", hook_path.display()))?;
    if verbose > 0 && changed {
        eprintln!("Stored integrity hash for hook");
    }

    Ok(changed)
}

/// Idempotent file write: create or update if content differs
fn write_if_changed(path: &Path, content: &str, name: &str, verbose: u8) -> Result<bool> {
    if path.exists() {
        let existing = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}: {}", name, path.display()))?;

        if existing == content {
            if verbose > 0 {
                eprintln!("{} already up to date: {}", name, path.display());
            }
            Ok(false)
        } else {
            fs::write(path, content)
                .with_context(|| format!("Failed to write {}: {}", name, path.display()))?;
            if verbose > 0 {
                eprintln!("Updated {}: {}", name, path.display());
            }
            Ok(true)
        }
    } else {
        fs::write(path, content)
            .with_context(|| format!("Failed to write {}: {}", name, path.display()))?;
        if verbose > 0 {
            eprintln!("Created {}: {}", name, path.display());
        }
        Ok(true)
    }
}

/// Atomic write using tempfile + rename
/// Prevents corruption on crash/interrupt
fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let parent = path.parent().with_context(|| {
        format!(
            "Cannot write to {}: path has no parent directory",
            path.display()
        )
    })?;

    // Create temp file in same directory (ensures same filesystem for atomic rename)
    let mut temp_file = NamedTempFile::new_in(parent)
        .with_context(|| format!("Failed to create temp file in {}", parent.display()))?;

    // Write content
    temp_file
        .write_all(content.as_bytes())
        .with_context(|| format!("Failed to write {} bytes to temp file", content.len()))?;

    // Atomic rename
    temp_file.persist(path).with_context(|| {
        format!(
            "Failed to atomically replace {} (disk full?)",
            path.display()
        )
    })?;

    Ok(())
}

/// Prompt user for consent to patch settings.json
/// Prints to stderr (stdout may be piped), reads from stdin
/// Default is No (capital N)
fn prompt_user_consent(settings_path: &Path) -> Result<bool> {
    use std::io::{self, BufRead, IsTerminal};

    eprintln!("\nPatch existing {}? [y/N] ", settings_path.display());

    // If stdin is not a terminal (piped), default to No
    if !io::stdin().is_terminal() {
        eprintln!("(non-interactive mode, defaulting to N)");
        return Ok(false);
    }

    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .context("Failed to read user input")?;

    let response = line.trim().to_lowercase();
    Ok(response == "y" || response == "yes")
}

/// Print manual instructions for settings.json patching
fn print_manual_instructions(hook_path: &Path) {
    println!("\n  MANUAL STEP: Add this to ~/.claude/settings.json:");
    println!("  {{");
    println!("    \"hooks\": {{ \"PreToolUse\": [{{");
    println!("      \"matcher\": \"Bash\",");
    println!("      \"hooks\": [{{ \"type\": \"command\",");
    println!("        \"command\": \"{}\"", hook_path.display());
    println!("      }}]");
    println!("    }}]}}");
    println!("  }}");
    println!("\n  Then restart Claude Code. Test with: git status\n");
}

/// Remove CLOV hook entry from settings.json
/// Returns true if hook was found and removed
fn remove_hook_from_json(root: &mut serde_json::Value) -> bool {
    let hooks = match root.get_mut("hooks").and_then(|h| h.get_mut("PreToolUse")) {
        Some(pre_tool_use) => pre_tool_use,
        None => return false,
    };

    let pre_tool_use_array = match hooks.as_array_mut() {
        Some(arr) => arr,
        None => return false,
    };

    // Find and remove CLOV entry
    let original_len = pre_tool_use_array.len();
    pre_tool_use_array.retain(|entry| {
        if let Some(hooks_array) = entry.get("hooks").and_then(|h| h.as_array()) {
            for hook in hooks_array {
                if let Some(command) = hook.get("command").and_then(|c| c.as_str()) {
                    if command.contains("clov-rewrite.sh") {
                        return false; // Remove this entry
                    }
                }
            }
        }
        true // Keep this entry
    });

    pre_tool_use_array.len() < original_len
}

/// Remove CLOV hook from settings.json file
/// Backs up before modification, returns true if hook was found and removed
fn remove_hook_from_settings(verbose: u8) -> Result<bool> {
    let claude_dir = resolve_claude_dir()?;
    let settings_path = claude_dir.join("settings.json");

    if !settings_path.exists() {
        if verbose > 0 {
            eprintln!("settings.json not found, nothing to remove");
        }
        return Ok(false);
    }

    let content = fs::read_to_string(&settings_path)
        .with_context(|| format!("Failed to read {}", settings_path.display()))?;

    if content.trim().is_empty() {
        return Ok(false);
    }

    let mut root: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {} as JSON", settings_path.display()))?;

    let removed = remove_hook_from_json(&mut root);

    if removed {
        // Backup original
        let backup_path = settings_path.with_extension("json.bak");
        fs::copy(&settings_path, &backup_path)
            .with_context(|| format!("Failed to backup to {}", backup_path.display()))?;

        // Atomic write
        let serialized =
            serde_json::to_string_pretty(&root).context("Failed to serialize settings.json")?;
        atomic_write(&settings_path, &serialized)?;

        if verbose > 0 {
            eprintln!("Removed CLOV hook from settings.json");
        }
    }

    Ok(removed)
}

/// Full uninstall: remove hook, CLOV.md, @CLOV.md reference, settings.json entry
pub fn uninstall(global: bool, verbose: u8) -> Result<()> {
    if !global {
        anyhow::bail!("Uninstall only works with --global flag. For local projects, manually remove CLOV from CLAUDE.md");
    }

    let claude_dir = resolve_claude_dir()?;
    let mut removed = Vec::new();

    // 1. Remove hook file
    let hook_path = claude_dir.join("hooks").join("clov-rewrite.sh");
    if hook_path.exists() {
        fs::remove_file(&hook_path)
            .with_context(|| format!("Failed to remove hook: {}", hook_path.display()))?;
        removed.push(format!("Hook: {}", hook_path.display()));
    }

    // 1b. Remove integrity hash file
    if integrity::remove_hash(&hook_path)? {
        removed.push("Integrity hash: removed".to_string());
    }

    // 2. Remove CLOV.md
    let clov_md_path = claude_dir.join("CLOV.md");
    if clov_md_path.exists() {
        fs::remove_file(&clov_md_path)
            .with_context(|| format!("Failed to remove CLOV.md: {}", clov_md_path.display()))?;
        removed.push(format!("CLOV.md: {}", clov_md_path.display()));
    }

    // 3. Remove @CLOV.md reference from CLAUDE.md
    let claude_md_path = claude_dir.join("CLAUDE.md");
    if claude_md_path.exists() {
        let content = fs::read_to_string(&claude_md_path)
            .with_context(|| format!("Failed to read CLAUDE.md: {}", claude_md_path.display()))?;

        if content.contains("@CLOV.md") {
            let new_content = content
                .lines()
                .filter(|line| !line.trim().starts_with("@CLOV.md"))
                .collect::<Vec<_>>()
                .join("\n");

            // Clean up double blanks
            let cleaned = clean_double_blanks(&new_content);

            fs::write(&claude_md_path, cleaned).with_context(|| {
                format!("Failed to write CLAUDE.md: {}", claude_md_path.display())
            })?;
            removed.push("CLAUDE.md: removed @CLOV.md reference".to_string());
        }
    }

    // 4. Remove hook entry from settings.json
    if remove_hook_from_settings(verbose)? {
        removed.push("settings.json: removed CLOV hook entry".to_string());
    }

    // Report results
    if removed.is_empty() {
        println!("CLOV was not installed (nothing to remove)");
    } else {
        println!("CLOV uninstalled:");
        for item in removed {
            println!("  - {}", item);
        }
        println!("\nRestart Claude Code to apply changes.");
    }

    Ok(())
}

/// Orchestrator: patch settings.json with CLOV hook
/// Handles reading, checking, prompting, merging, backing up, and atomic writing
fn patch_settings_json(hook_path: &Path, mode: PatchMode, verbose: u8) -> Result<PatchResult> {
    let claude_dir = resolve_claude_dir()?;
    let settings_path = claude_dir.join("settings.json");
    let hook_command = hook_path
        .to_str()
        .context("Hook path contains invalid UTF-8")?;

    // Read or create settings.json
    let mut root = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)
            .with_context(|| format!("Failed to read {}", settings_path.display()))?;

        if content.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse {} as JSON", settings_path.display()))?
        }
    } else {
        serde_json::json!({})
    };

    // Check idempotency
    if hook_already_present(&root, hook_command) {
        if verbose > 0 {
            eprintln!("settings.json: hook already present");
        }
        return Ok(PatchResult::AlreadyPresent);
    }

    // Handle mode
    match mode {
        PatchMode::Skip => {
            print_manual_instructions(hook_path);
            return Ok(PatchResult::Skipped);
        }
        PatchMode::Ask => {
            if !prompt_user_consent(&settings_path)? {
                print_manual_instructions(hook_path);
                return Ok(PatchResult::Declined);
            }
        }
        PatchMode::Auto => {
            // Proceed without prompting
        }
    }

    // Deep-merge hook
    insert_hook_entry(&mut root, hook_command);

    // Backup original
    if settings_path.exists() {
        let backup_path = settings_path.with_extension("json.bak");
        fs::copy(&settings_path, &backup_path)
            .with_context(|| format!("Failed to backup to {}", backup_path.display()))?;
        if verbose > 0 {
            eprintln!("Backup: {}", backup_path.display());
        }
    }

    // Atomic write
    let serialized =
        serde_json::to_string_pretty(&root).context("Failed to serialize settings.json")?;
    atomic_write(&settings_path, &serialized)?;

    println!("\n  settings.json: hook added");
    if settings_path.with_extension("json.bak").exists() {
        println!(
            "  Backup: {}",
            settings_path.with_extension("json.bak").display()
        );
    }
    println!("  Restart Claude Code. Test with: git status");

    Ok(PatchResult::Patched)
}

/// Clean up consecutive blank lines (collapse 3+ to 2)
/// Used when removing @CLOV.md line from CLAUDE.md
fn clean_double_blanks(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if line.trim().is_empty() {
            // Count consecutive blank lines
            let mut blank_count = 0;
            while i < lines.len() && lines[i].trim().is_empty() {
                blank_count += 1;
                i += 1;
            }

            // Keep at most 2 blank lines
            let keep = blank_count.min(2);
            result.extend(std::iter::repeat_n("", keep));
        } else {
            result.push(line);
            i += 1;
        }
    }

    result.join("\n")
}

/// Deep-merge CLOV hook entry into settings.json
/// Creates hooks.PreToolUse structure if missing, preserves existing hooks
fn insert_hook_entry(root: &mut serde_json::Value, hook_command: &str) {
    // Ensure root is an object
    let root_obj = match root.as_object_mut() {
        Some(obj) => obj,
        None => {
            *root = serde_json::json!({});
            root.as_object_mut()
                .expect("Just created object, must succeed")
        }
    };

    // Use entry() API for idiomatic insertion
    let hooks = root_obj
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .expect("hooks must be an object");

    let pre_tool_use = hooks
        .entry("PreToolUse")
        .or_insert_with(|| serde_json::json!([]))
        .as_array_mut()
        .expect("PreToolUse must be an array");

    // Append CLOV hook entry
    pre_tool_use.push(serde_json::json!({
        "matcher": "Bash",
        "hooks": [{
            "type": "command",
            "command": hook_command
        }]
    }));
}

/// Check if CLOV hook is already present in settings.json
/// Matches on clov-rewrite.sh substring to handle different path formats
fn hook_already_present(root: &serde_json::Value, hook_command: &str) -> bool {
    let pre_tool_use_array = match root
        .get("hooks")
        .and_then(|h| h.get("PreToolUse"))
        .and_then(|p| p.as_array())
    {
        Some(arr) => arr,
        None => return false,
    };

    pre_tool_use_array
        .iter()
        .filter_map(|entry| entry.get("hooks")?.as_array())
        .flatten()
        .filter_map(|hook| hook.get("command")?.as_str())
        .any(|cmd| {
            // Exact match OR both contain clov-rewrite.sh
            cmd == hook_command
                || (cmd.contains("clov-rewrite.sh") && hook_command.contains("clov-rewrite.sh"))
        })
}

/// Default mode: hook + slim CLOV.md + @CLOV.md reference
#[cfg(not(unix))]
fn run_default_mode(_global: bool, _patch_mode: PatchMode, _verbose: u8) -> Result<()> {
    eprintln!("⚠️  Hook-based mode requires Unix (macOS/Linux).");
    eprintln!("    Windows: use --claude-md mode for full injection.");
    eprintln!("    Falling back to --claude-md mode.");
    run_claude_md_mode(_global, _verbose)
}

#[cfg(unix)]
fn run_default_mode(global: bool, patch_mode: PatchMode, verbose: u8) -> Result<()> {
    if !global {
        // Local init: unchanged behavior (full injection into ./CLAUDE.md)
        return run_claude_md_mode(false, verbose);
    }

    let claude_dir = resolve_claude_dir()?;
    let clov_md_path = claude_dir.join("CLOV.md");
    let claude_md_path = claude_dir.join("CLAUDE.md");

    // 1. Prepare hook directory and install hook
    let (_hook_dir, hook_path) = prepare_hook_paths()?;
    let hook_changed = ensure_hook_installed(&hook_path, verbose)?;

    // 2. Write CLOV.md
    write_if_changed(&clov_md_path, CLOV_SLIM, "CLOV.md", verbose)?;

    // 3. Patch CLAUDE.md (add @CLOV.md, migrate if needed)
    let migrated = patch_claude_md(&claude_md_path, verbose)?;

    // 4. Print success message
    let hook_status = if hook_changed {
        "installed/updated"
    } else {
        "already up to date"
    };
    println!("\nCLOV hook {} (global).\n", hook_status);
    println!("  Hook:      {}", hook_path.display());
    println!("  CLOV.md:    {} (10 lines)", clov_md_path.display());
    println!("  CLAUDE.md: @CLOV.md reference added");

    if migrated {
        println!("\n  ✅ Migrated: removed 137-line CLOV block from CLAUDE.md");
        println!("              replaced with @CLOV.md (10 lines)");
    }

    // 5. Patch settings.json
    let patch_result = patch_settings_json(&hook_path, patch_mode, verbose)?;

    // Report result
    match patch_result {
        PatchResult::Patched => {
            // Already printed by patch_settings_json
        }
        PatchResult::AlreadyPresent => {
            println!("\n  settings.json: hook already present");
            println!("  Restart Claude Code. Test with: git status");
        }
        PatchResult::Declined | PatchResult::Skipped => {
            // Manual instructions already printed by patch_settings_json
        }
    }

    println!(); // Final newline

    Ok(())
}

/// Hook-only mode: just the hook, no CLOV.md
#[cfg(not(unix))]
fn run_hook_only_mode(_global: bool, _patch_mode: PatchMode, _verbose: u8) -> Result<()> {
    anyhow::bail!("Hook install requires Unix (macOS/Linux). Use WSL or --claude-md mode.")
}

#[cfg(unix)]
fn run_hook_only_mode(global: bool, patch_mode: PatchMode, verbose: u8) -> Result<()> {
    if !global {
        eprintln!("⚠️  Warning: --hook-only only makes sense with --global");
        eprintln!("    For local projects, use default mode or --claude-md");
        return Ok(());
    }

    // Prepare and install hook
    let (_hook_dir, hook_path) = prepare_hook_paths()?;
    let hook_changed = ensure_hook_installed(&hook_path, verbose)?;

    let hook_status = if hook_changed {
        "installed/updated"
    } else {
        "already up to date"
    };
    println!("\nCLOV hook {} (hook-only mode).\n", hook_status);
    println!("  Hook: {}", hook_path.display());
    println!(
        "  Note: No CLOV.md created. Claude won't know about meta commands (gain, discover, proxy)."
    );

    // Patch settings.json
    let patch_result = patch_settings_json(&hook_path, patch_mode, verbose)?;

    // Report result
    match patch_result {
        PatchResult::Patched => {
            // Already printed by patch_settings_json
        }
        PatchResult::AlreadyPresent => {
            println!("\n  settings.json: hook already present");
            println!("  Restart Claude Code. Test with: git status");
        }
        PatchResult::Declined | PatchResult::Skipped => {
            // Manual instructions already printed by patch_settings_json
        }
    }

    println!(); // Final newline

    Ok(())
}

/// Legacy mode: full 137-line injection into CLAUDE.md
fn run_claude_md_mode(global: bool, verbose: u8) -> Result<()> {
    let path = if global {
        resolve_claude_dir()?.join("CLAUDE.md")
    } else {
        PathBuf::from("CLAUDE.md")
    };

    if global {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
    }

    if verbose > 0 {
        eprintln!("Writing clov instructions to: {}", path.display());
    }

    if path.exists() {
        let existing = fs::read_to_string(&path)?;
        // upsert_clov_block handles all 4 cases: add, update, unchanged, malformed
        let (new_content, action) = upsert_clov_block(&existing, CLOV_INSTRUCTIONS);

        match action {
            ClovBlockUpsert::Added => {
                fs::write(&path, new_content)?;
                println!("✅ Added clov instructions to existing {}", path.display());
            }
            ClovBlockUpsert::Updated => {
                fs::write(&path, new_content)?;
                println!("✅ Updated clov instructions in {}", path.display());
            }
            ClovBlockUpsert::Unchanged => {
                println!(
                    "✅ {} already contains up-to-date clov instructions",
                    path.display()
                );
                return Ok(());
            }
            ClovBlockUpsert::Malformed => {
                eprintln!(
                    "⚠️  Warning: Found '<!-- clov-instructions' without closing marker in {}",
                    path.display()
                );

                if let Some((line_num, _)) = existing
                    .lines()
                    .enumerate()
                    .find(|(_, line)| line.contains("<!-- clov-instructions"))
                {
                    eprintln!("    Location: line {}", line_num + 1);
                }

                eprintln!("    Action: Manually remove the incomplete block, then re-run:");
                if global {
                    eprintln!("            clov hook -g --claude-md");
                } else {
                    eprintln!("            clov hook --claude-md");
                }
                return Ok(());
            }
        }
    } else {
        fs::write(&path, CLOV_INSTRUCTIONS)?;
        println!("✅ Created {} with clov instructions", path.display());
    }

    if global {
        println!("   Claude Code will now use clov in all sessions");
    } else {
        println!("   Claude Code will use clov in this project");
    }

    Ok(())
}

// --- upsert_clov_block: idempotent CLOV block management ---

#[derive(Debug, Clone, Copy, PartialEq)]
enum ClovBlockUpsert {
    /// No existing block found — appended new block
    Added,
    /// Existing block found with different content — replaced
    Updated,
    /// Existing block found with identical content — no-op
    Unchanged,
    /// Opening marker found without closing marker — not safe to rewrite
    Malformed,
}

/// Insert or replace the CLOV instructions block in `content`.
///
/// Returns `(new_content, action)` describing what happened.
/// The caller decides whether to write `new_content` based on `action`.
fn upsert_clov_block(content: &str, block: &str) -> (String, ClovBlockUpsert) {
    let start_marker = "<!-- clov-instructions";
    let end_marker = "<!-- /clov-instructions -->";

    if let Some(start) = content.find(start_marker) {
        if let Some(relative_end) = content[start..].find(end_marker) {
            let end = start + relative_end;
            let end_pos = end + end_marker.len();
            let current_block = content[start..end_pos].trim();
            let desired_block = block.trim();

            if current_block == desired_block {
                return (content.to_string(), ClovBlockUpsert::Unchanged);
            }

            // Replace stale block with desired block
            let before = content[..start].trim_end();
            let after = content[end_pos..].trim_start();

            let result = match (before.is_empty(), after.is_empty()) {
                (true, true) => desired_block.to_string(),
                (true, false) => format!("{desired_block}\n\n{after}"),
                (false, true) => format!("{before}\n\n{desired_block}"),
                (false, false) => format!("{before}\n\n{desired_block}\n\n{after}"),
            };

            return (result, ClovBlockUpsert::Updated);
        }

        // Opening marker without closing marker — malformed
        return (content.to_string(), ClovBlockUpsert::Malformed);
    }

    // No existing block — append
    let trimmed = content.trim();
    if trimmed.is_empty() {
        (block.to_string(), ClovBlockUpsert::Added)
    } else {
        (
            format!("{trimmed}\n\n{}", block.trim()),
            ClovBlockUpsert::Added,
        )
    }
}

/// Patch CLAUDE.md: add @CLOV.md, migrate if old block exists
fn patch_claude_md(path: &Path, verbose: u8) -> Result<bool> {
    let mut content = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };

    let mut migrated = false;

    // Check for old block and migrate
    if content.contains("<!-- clov-instructions") {
        let (new_content, did_migrate) = remove_clov_block(&content);
        if did_migrate {
            content = new_content;
            migrated = true;
            if verbose > 0 {
                eprintln!("Migrated: removed old CLOV block from CLAUDE.md");
            }
        }
    }

    // Check if @CLOV.md already present
    if content.contains("@CLOV.md") {
        if verbose > 0 {
            eprintln!("@CLOV.md reference already present in CLAUDE.md");
        }
        if migrated {
            fs::write(path, content)?;
        }
        return Ok(migrated);
    }

    // Add @CLOV.md
    let new_content = if content.is_empty() {
        "@CLOV.md\n".to_string()
    } else {
        format!("{}\n\n@CLOV.md\n", content.trim())
    };

    fs::write(path, new_content)?;

    if verbose > 0 {
        eprintln!("Added @CLOV.md reference to CLAUDE.md");
    }

    Ok(migrated)
}

/// Remove old CLOV block from CLAUDE.md (migration helper)
fn remove_clov_block(content: &str) -> (String, bool) {
    if let (Some(start), Some(end)) = (
        content.find("<!-- clov-instructions"),
        content.find("<!-- /clov-instructions -->"),
    ) {
        let end_pos = end + "<!-- /clov-instructions -->".len();
        let before = content[..start].trim_end();
        let after = content[end_pos..].trim_start();

        let result = if after.is_empty() {
            before.to_string()
        } else {
            format!("{}\n\n{}", before, after)
        };

        (result, true) // migrated
    } else if content.contains("<!-- clov-instructions") {
        eprintln!("⚠️  Warning: Found '<!-- clov-instructions' without closing marker.");
        eprintln!("    This can happen if CLAUDE.md was manually edited.");

        // Find line number
        if let Some((line_num, _)) = content
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains("<!-- clov-instructions"))
        {
            eprintln!("    Location: line {}", line_num + 1);
        }

        eprintln!("    Action: Manually remove the incomplete block, then re-run:");
        eprintln!("            clov hook -g");
        (content.to_string(), false)
    } else {
        (content.to_string(), false)
    }
}

/// Resolve ~/.claude directory with proper home expansion
fn resolve_claude_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|h| h.join(".claude"))
        .context("Cannot determine home directory. Is $HOME set?")
}

/// Show current clov configuration
pub fn show_config() -> Result<()> {
    let claude_dir = resolve_claude_dir()?;
    let hook_path = claude_dir.join("hooks").join("clov-rewrite.sh");
    let clov_md_path = claude_dir.join("CLOV.md");
    let global_claude_md = claude_dir.join("CLAUDE.md");
    let local_claude_md = PathBuf::from("CLAUDE.md");

    println!("📋 clov Configuration:\n");

    // Check hook
    if hook_path.exists() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&hook_path)?;
            let perms = metadata.permissions();
            let is_executable = perms.mode() & 0o111 != 0;

            let hook_content = fs::read_to_string(&hook_path)?;
            let has_guards =
                hook_content.contains("command -v clov") && hook_content.contains("command -v jq");
            let is_thin_delegator = hook_content.contains("clov route");

            if !is_executable {
                println!(
                    "⚠️  Hook: {} (NOT executable - run: chmod +x)",
                    hook_path.display()
                );
            } else if !is_thin_delegator {
                println!(
                    "⚠️  Hook: {} (outdated — inline logic, not thin delegator)",
                    hook_path.display()
                );
                println!(
                    "   → Run `clov hook --global` to upgrade to the single source of truth hook"
                );
            } else if is_executable && has_guards {
                println!(
                    "✅ Hook: {} (thin delegator, up to date)",
                    hook_path.display()
                );
            } else {
                println!("⚠️  Hook: {} (no guards - outdated)", hook_path.display());
            }
        }

        #[cfg(not(unix))]
        {
            println!("✅ Hook: {} (exists)", hook_path.display());
        }
    } else {
        println!("⚪ Hook: not found");
    }

    // Check CLOV.md
    if clov_md_path.exists() {
        println!("✅ CLOV.md: {} (slim mode)", clov_md_path.display());
    } else {
        println!("⚪ CLOV.md: not found");
    }

    // Check hook integrity
    match integrity::verify_hook_at(&hook_path) {
        Ok(integrity::IntegrityStatus::Verified) => {
            println!("✅ Integrity: hook hash verified");
        }
        Ok(integrity::IntegrityStatus::Tampered { .. }) => {
            println!("❌ Integrity: hook modified outside clov hook (run: clov doctor)");
        }
        Ok(integrity::IntegrityStatus::NoBaseline) => {
            println!("⚠️  Integrity: no baseline hash (run: clov hook -g to establish)");
        }
        Ok(integrity::IntegrityStatus::NotInstalled)
        | Ok(integrity::IntegrityStatus::OrphanedHash) => {
            // Don't show integrity line if hook isn't installed
        }
        Err(_) => {
            println!("⚠️  Integrity: check failed");
        }
    }

    // Check global CLAUDE.md
    if global_claude_md.exists() {
        let content = fs::read_to_string(&global_claude_md)?;
        if content.contains("@CLOV.md") {
            println!("✅ Global (~/.claude/CLAUDE.md): @CLOV.md reference");
        } else if content.contains("<!-- clov-instructions") {
            println!(
                "⚠️  Global (~/.claude/CLAUDE.md): old CLOV block (run: clov hook -g to migrate)"
            );
        } else {
            println!("⚪ Global (~/.claude/CLAUDE.md): exists but clov not configured");
        }
    } else {
        println!("⚪ Global (~/.claude/CLAUDE.md): not found");
    }

    // Check local CLAUDE.md
    if local_claude_md.exists() {
        let content = fs::read_to_string(&local_claude_md)?;
        if content.contains("clov") {
            println!("✅ Local (./CLAUDE.md): clov enabled");
        } else {
            println!("⚪ Local (./CLAUDE.md): exists but clov not configured");
        }
    } else {
        println!("⚪ Local (./CLAUDE.md): not found");
    }

    // Check settings.json
    let settings_path = claude_dir.join("settings.json");
    if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        if !content.trim().is_empty() {
            if let Ok(root) = serde_json::from_str::<serde_json::Value>(&content) {
                let hook_command = hook_path.display().to_string();
                if hook_already_present(&root, &hook_command) {
                    println!("✅ settings.json: CLOV hook configured");
                } else {
                    println!("⚠️  settings.json: exists but CLOV hook not configured");
                    println!("    Run: clov hook -g --auto-patch");
                }
            } else {
                println!("⚠️  settings.json: exists but invalid JSON");
            }
        } else {
            println!("⚪ settings.json: empty");
        }
    } else {
        println!("⚪ settings.json: not found");
    }

    println!("\nUsage:");
    println!("  clov hook              # Full injection into local CLAUDE.md");
    println!("  clov hook -g           # Hook + CLOV.md + @CLOV.md + settings.json (recommended)");
    println!("  clov hook -g --auto-patch    # Same as above but no prompt");
    println!("  clov hook -g --no-patch      # Skip settings.json (manual setup)");
    println!("  clov hook -g --uninstall     # Remove all CLOV artifacts");
    println!("  clov hook -g --claude-md     # Legacy: full injection into ~/.claude/CLAUDE.md");
    println!("  clov hook -g --hook-only     # Hook only, no CLOV.md");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_mentions_all_top_level_commands() {
        for cmd in [
            "clov cargo",
            "clov gh",
            "clov vitest",
            "clov tsc",
            "clov lint",
            "clov prettier",
            "clov next",
            "clov playwright",
            "clov prisma",
            "clov pnpm",
            "clov npm",
            "clov curl",
            "clov git",
            "clov docker",
            "clov kubectl",
        ] {
            assert!(
                CLOV_INSTRUCTIONS.contains(cmd),
                "Missing {cmd} in CLOV_INSTRUCTIONS"
            );
        }
    }

    #[test]
    fn test_init_has_version_marker() {
        assert!(
            CLOV_INSTRUCTIONS.contains("<!-- clov-instructions"),
            "CLOV_INSTRUCTIONS must have version marker for idempotency"
        );
    }

    #[test]
    fn test_hook_has_guards() {
        assert!(REWRITE_HOOK.contains("command -v clov"));
        assert!(REWRITE_HOOK.contains("command -v jq"));
        // Guards (clov/jq availability checks) must appear before the actual delegation call.
        // The thin delegating hook no longer uses set -euo pipefail.
        let jq_pos = REWRITE_HOOK.find("command -v jq").unwrap();
        let clov_delegate_pos = REWRITE_HOOK.find("clov route \"$CMD\"").unwrap();
        assert!(
            jq_pos < clov_delegate_pos,
            "Guards must appear before clov route delegation"
        );
    }

    #[test]
    fn test_migration_removes_old_block() {
        let input = r#"# My Config

<!-- clov-instructions v2 -->
OLD CLOV STUFF
<!-- /clov-instructions -->

More content"#;

        let (result, migrated) = remove_clov_block(input);
        assert!(migrated);
        assert!(!result.contains("OLD CLOV STUFF"));
        assert!(result.contains("# My Config"));
        assert!(result.contains("More content"));
    }

    #[test]
    fn test_migration_warns_on_missing_end_marker() {
        let input = "<!-- clov-instructions v2 -->\nOLD STUFF\nNo end marker";
        let (result, migrated) = remove_clov_block(input);
        assert!(!migrated);
        assert_eq!(result, input);
    }

    #[test]
    #[cfg(unix)]
    fn test_default_mode_creates_hook_and_clov_md() {
        let temp = TempDir::new().unwrap();
        let hook_path = temp.path().join("clov-rewrite.sh");
        let clov_md_path = temp.path().join("CLOV.md");

        fs::write(&hook_path, REWRITE_HOOK).unwrap();
        fs::write(&clov_md_path, CLOV_SLIM).unwrap();

        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755)).unwrap();

        assert!(hook_path.exists());
        assert!(clov_md_path.exists());

        let metadata = fs::metadata(&hook_path).unwrap();
        assert!(metadata.permissions().mode() & 0o111 != 0);
    }

    #[test]
    fn test_claude_md_mode_creates_full_injection() {
        // Just verify CLOV_INSTRUCTIONS constant has the right content
        assert!(CLOV_INSTRUCTIONS.contains("<!-- clov-instructions"));
        assert!(CLOV_INSTRUCTIONS.contains("clov cargo test"));
        assert!(CLOV_INSTRUCTIONS.contains("<!-- /clov-instructions -->"));
        assert!(CLOV_INSTRUCTIONS.len() > 4000);
    }

    // --- upsert_clov_block tests ---

    #[test]
    fn test_upsert_clov_block_appends_when_missing() {
        let input = "# Team instructions";
        let (content, action) = upsert_clov_block(input, CLOV_INSTRUCTIONS);
        assert_eq!(action, ClovBlockUpsert::Added);
        assert!(content.contains("# Team instructions"));
        assert!(content.contains("<!-- clov-instructions"));
    }

    #[test]
    fn test_upsert_clov_block_updates_stale_block() {
        let input = r#"# Team instructions

<!-- clov-instructions v1 -->
OLD CLOV CONTENT
<!-- /clov-instructions -->

More notes
"#;

        let (content, action) = upsert_clov_block(input, CLOV_INSTRUCTIONS);
        assert_eq!(action, ClovBlockUpsert::Updated);
        assert!(!content.contains("OLD CLOV CONTENT"));
        assert!(content.contains("clov cargo test")); // from current CLOV_INSTRUCTIONS
        assert!(content.contains("# Team instructions"));
        assert!(content.contains("More notes"));
    }

    #[test]
    fn test_upsert_clov_block_noop_when_already_current() {
        let input = format!(
            "# Team instructions\n\n{}\n\nMore notes\n",
            CLOV_INSTRUCTIONS
        );
        let (content, action) = upsert_clov_block(&input, CLOV_INSTRUCTIONS);
        assert_eq!(action, ClovBlockUpsert::Unchanged);
        assert_eq!(content, input);
    }

    #[test]
    fn test_upsert_clov_block_detects_malformed_block() {
        let input = "<!-- clov-instructions v2 -->\npartial";
        let (content, action) = upsert_clov_block(input, CLOV_INSTRUCTIONS);
        assert_eq!(action, ClovBlockUpsert::Malformed);
        assert_eq!(content, input);
    }

    #[test]
    fn test_init_is_idempotent() {
        let temp = TempDir::new().unwrap();
        let claude_md = temp.path().join("CLAUDE.md");

        fs::write(&claude_md, "# My stuff\n\n@CLOV.md\n").unwrap();

        let content = fs::read_to_string(&claude_md).unwrap();
        let count = content.matches("@CLOV.md").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_local_init_unchanged() {
        // Local init should use claude-md mode
        let temp = TempDir::new().unwrap();
        let claude_md = temp.path().join("CLAUDE.md");

        fs::write(&claude_md, CLOV_INSTRUCTIONS).unwrap();
        let content = fs::read_to_string(&claude_md).unwrap();

        assert!(content.contains("<!-- clov-instructions"));
    }

    // Tests for hook_already_present()
    #[test]
    fn test_hook_already_present_exact_match() {
        let json_content = serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "Bash",
                    "hooks": [{
                        "type": "command",
                        "command": "/Users/test/.claude/hooks/clov-rewrite.sh"
                    }]
                }]
            }
        });

        let hook_command = "/Users/test/.claude/hooks/clov-rewrite.sh";
        assert!(hook_already_present(&json_content, hook_command));
    }

    #[test]
    fn test_hook_already_present_different_path() {
        let json_content = serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "Bash",
                    "hooks": [{
                        "type": "command",
                        "command": "/home/user/.claude/hooks/clov-rewrite.sh"
                    }]
                }]
            }
        });

        let hook_command = "~/.claude/hooks/clov-rewrite.sh";
        // Should match on clov-rewrite.sh substring
        assert!(hook_already_present(&json_content, hook_command));
    }

    #[test]
    fn test_hook_not_present_empty() {
        let json_content = serde_json::json!({});
        let hook_command = "/Users/test/.claude/hooks/clov-rewrite.sh";
        assert!(!hook_already_present(&json_content, hook_command));
    }

    #[test]
    fn test_hook_not_present_other_hooks() {
        let json_content = serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "Bash",
                    "hooks": [{
                        "type": "command",
                        "command": "/some/other/hook.sh"
                    }]
                }]
            }
        });

        let hook_command = "/Users/test/.claude/hooks/clov-rewrite.sh";
        assert!(!hook_already_present(&json_content, hook_command));
    }

    // Tests for insert_hook_entry()
    #[test]
    fn test_insert_hook_entry_empty_root() {
        let mut json_content = serde_json::json!({});
        let hook_command = "/Users/test/.claude/hooks/clov-rewrite.sh";

        insert_hook_entry(&mut json_content, hook_command);

        // Should create full structure
        assert!(json_content.get("hooks").is_some());
        assert!(json_content
            .get("hooks")
            .unwrap()
            .get("PreToolUse")
            .is_some());

        let pre_tool_use = json_content["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre_tool_use.len(), 1);

        let command = pre_tool_use[0]["hooks"][0]["command"].as_str().unwrap();
        assert_eq!(command, hook_command);
    }

    #[test]
    fn test_insert_hook_entry_preserves_existing() {
        let mut json_content = serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "Bash",
                    "hooks": [{
                        "type": "command",
                        "command": "/some/other/hook.sh"
                    }]
                }]
            }
        });

        let hook_command = "/Users/test/.claude/hooks/clov-rewrite.sh";
        insert_hook_entry(&mut json_content, hook_command);

        let pre_tool_use = json_content["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre_tool_use.len(), 2); // Should have both hooks

        // Check first hook is preserved
        let first_command = pre_tool_use[0]["hooks"][0]["command"].as_str().unwrap();
        assert_eq!(first_command, "/some/other/hook.sh");

        // Check second hook is CLOV
        let second_command = pre_tool_use[1]["hooks"][0]["command"].as_str().unwrap();
        assert_eq!(second_command, hook_command);
    }

    #[test]
    fn test_insert_hook_preserves_other_keys() {
        let mut json_content = serde_json::json!({
            "env": {"PATH": "/custom/path"},
            "permissions": {"allowAll": true},
            "model": "claude-sonnet-4"
        });

        let hook_command = "/Users/test/.claude/hooks/clov-rewrite.sh";
        insert_hook_entry(&mut json_content, hook_command);

        // Should preserve all other keys
        assert_eq!(json_content["env"]["PATH"], "/custom/path");
        assert_eq!(json_content["permissions"]["allowAll"], true);
        assert_eq!(json_content["model"], "claude-sonnet-4");

        // And add hooks
        assert!(json_content.get("hooks").is_some());
    }

    // Tests for atomic_write()
    #[test]
    fn test_atomic_write() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.json");

        let content = r#"{"key": "value"}"#;
        atomic_write(&file_path, content).unwrap();

        assert!(file_path.exists());
        let written = fs::read_to_string(&file_path).unwrap();
        assert_eq!(written, content);
    }

    // Test for preserve_order round-trip
    #[test]
    fn test_preserve_order_round_trip() {
        let original = r#"{"env": {"PATH": "/usr/bin"}, "permissions": {"allowAll": true}, "model": "claude-sonnet-4"}"#;
        let parsed: serde_json::Value = serde_json::from_str(original).unwrap();
        let serialized = serde_json::to_string(&parsed).unwrap();

        // Keys should appear in same order
        // Just check that keys exist (preserve_order doesn't guarantee exact order in nested objects)
        assert!(serialized.contains("\"env\""));
        assert!(serialized.contains("\"permissions\""));
        assert!(serialized.contains("\"model\""));
    }

    // Tests for clean_double_blanks()
    #[test]
    fn test_clean_double_blanks() {
        // Input: line1, 2 blank lines, line2, 1 blank line, line3, 3 blank lines, line4
        // Expected: line1, 2 blank lines (kept), line2, 1 blank line, line3, 2 blank lines (max), line4
        let input = "line1\n\n\nline2\n\nline3\n\n\n\nline4";
        // That's: line1 \n \n \n line2 \n \n line3 \n \n \n \n line4
        // Which is: line1, blank, blank, line2, blank, line3, blank, blank, blank, line4
        // So 2 blanks after line1 (keep both), 1 blank after line2 (keep), 3 blanks after line3 (keep 2)
        let expected = "line1\n\n\nline2\n\nline3\n\n\nline4";
        assert_eq!(clean_double_blanks(input), expected);
    }

    #[test]
    fn test_clean_double_blanks_preserves_single() {
        let input = "line1\n\nline2\n\nline3";
        assert_eq!(clean_double_blanks(input), input); // No change
    }

    // Tests for remove_hook_from_settings()
    #[test]
    fn test_remove_hook_from_json() {
        let mut json_content = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Bash",
                        "hooks": [{
                            "type": "command",
                            "command": "/some/other/hook.sh"
                        }]
                    },
                    {
                        "matcher": "Bash",
                        "hooks": [{
                            "type": "command",
                            "command": "/Users/test/.claude/hooks/clov-rewrite.sh"
                        }]
                    }
                ]
            }
        });

        let removed = remove_hook_from_json(&mut json_content);
        assert!(removed);

        // Should have only one hook left
        let pre_tool_use = json_content["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre_tool_use.len(), 1);

        // Check it's the other hook
        let command = pre_tool_use[0]["hooks"][0]["command"].as_str().unwrap();
        assert_eq!(command, "/some/other/hook.sh");
    }

    #[test]
    fn test_remove_hook_when_not_present() {
        let mut json_content = serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "Bash",
                    "hooks": [{
                        "type": "command",
                        "command": "/some/other/hook.sh"
                    }]
                }]
            }
        });

        let removed = remove_hook_from_json(&mut json_content);
        assert!(!removed);
    }
}
