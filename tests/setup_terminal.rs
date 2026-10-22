//! Exercise real terminal detection, raw key navigation and hidden credentials on native OSes.
use portable_pty::{CommandBuilder, PtySize};
use std::{
    fs,
    io::{Read, Write},
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};
struct Terminal {
    child: Box<dyn portable_pty::Child + Send + Sync>,
    _master: Box<dyn portable_pty::MasterPty + Send>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    receiver: mpsc::Receiver<Vec<u8>>,
    output: String,
    cursor: usize,
}
impl Terminal {
    fn start(root: &std::path::Path, plain: bool) -> Self {
        Self::start_in(root, plain, None)
    }
    fn start_in(root: &std::path::Path, plain: bool, cwd: Option<&std::path::Path>) -> Self {
        let pair = portable_pty::native_pty_system()
            .openpty(PtySize {
                rows: 45,
                cols: 150,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_framechord"));
        cmd.args(["--color", "never", "init"]);
        cmd.arg(root);
        if let Some(cwd) = cwd {
            cmd.cwd(cwd);
        }
        cmd.env("TERM", if plain { "dumb" } else { "xterm-256color" });
        for key in [
            "OPENAI_API_KEY",
            "FISH_API_KEY",
            "GEMINI_API_KEY",
            "OPENROUTER_API_KEY",
            "ELEVENLABS_API_KEY",
        ] {
            cmd.env_remove(key);
        }
        let child = pair.slave.spawn_command(cmd).unwrap();
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().unwrap();
        let writer = Arc::new(Mutex::new(pair.master.take_writer().unwrap()));
        let responses = Arc::clone(&writer);
        let (tx, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut buf = [0; 4096];
            let mut query = Vec::new();
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                // ConPTY asks the hosting terminal for its initial cursor position.
                // A PTY driver must answer this just as an actual terminal does.
                for byte in &buf[..n] {
                    query.push(*byte);
                    if query.ends_with(b"\x1b[6n") {
                        let mut input = responses.lock().unwrap();
                        if input
                            .write_all(b"\x1b[1;1R")
                            .and_then(|_| input.flush())
                            .is_err()
                        {
                            return;
                        }
                    }
                    if query.len() > 4 {
                        query.remove(0);
                    }
                }
                if tx.send(buf[..n].to_vec()).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            _master: pair.master,
            writer,
            receiver,
            output: String::new(),
            cursor: 0,
        }
    }
    fn wait(&mut self, token: &str) {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            // ConPTY may insert cursor visibility/control sequences inside a prompt.
            // Match the text a user sees; retain raw output for masking/color assertions.
            let visible = console::strip_ansi_codes(&self.output);
            if let Some(i) = visible[self.cursor..].find(token) {
                self.cursor += i + token.len();
                return;
            }
            assert!(
                Instant::now() < deadline,
                "terminal did not show {token:?}; output: {}",
                self.output
            );
            if let Ok(bytes) = self.receiver.recv_timeout(Duration::from_millis(100)) {
                self.output.push_str(&String::from_utf8_lossy(&bytes));
            }
        }
    }
    fn send(&mut self, keys: &str) {
        // Let the prompt renderer finish switching terminal input mode before typing.
        // ConPTY on a busy Windows runner can drop keys typed sooner; a lost "Save" then
        // falls through to the menu default and the process never exits.
        std::thread::sleep(Duration::from_millis(if cfg!(windows) { 200 } else { 50 }));
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(keys.as_bytes()).unwrap();
        writer.flush().unwrap();
    }
    /// Waits for the process to exit. Returns as soon as it does; the generous bound only
    /// matters on a hang, and then the assertion shows what the terminal last displayed.
    fn exit_status(&mut self) -> portable_pty::ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(45);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status;
            }
            while let Ok(bytes) = self.receiver.try_recv() {
                self.output.push_str(&String::from_utf8_lossy(&bytes));
            }
            assert!(
                Instant::now() < deadline,
                "terminal did not exit; output: {}",
                console::strip_ansi_codes(&self.output)
            );
            std::thread::sleep(Duration::from_millis(25));
        }
    }
    fn success(&mut self) {
        let status = self.exit_status();
        assert!(
            status.success(),
            "terminal exit {}: {}",
            status.exit_code(),
            self.output
        );
    }
}
#[test]
fn colliding_executable_offers_another_folder_and_saves_an_unfinished_project_there() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    fs::write(root.join("framechord"), "preserve executable").unwrap();
    let mut p = Terminal::start_in(std::path::Path::new("."), false, Some(root));
    p.wait("Choose where to save your project");
    p.send("\r");
    p.wait("Project folder (relative to the current directory)");
    p.send("project with spaces\r");
    p.wait("Welcome to FrameChord");
    p.send("\r");
    p.wait("Setup checklist");
    p.send("Save unfinished setup and exit\r");
    p.wait("Save these reviewed choices?");
    assert!(!root.join("project with spaces").exists());
    p.send("Save\r");
    p.success();
    let project = root.join("project with spaces");
    assert!(project.join("framechord/config.yaml").is_file());
    assert!(project.join(".agents/skills/framechord/SKILL.md").is_file());
    assert!(project.join(".claude/skills/framechord/SKILL.md").is_file());
    assert_eq!(
        fs::read_to_string(root.join("framechord")).unwrap(),
        "preserve executable"
    );
    assert!(framechord::setup::status(&project).unwrap()["setup_complete"] == false);
    assert!(!p.output.contains("local_io"));
}
impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
#[test]
fn existing_google_key_finishes_default_setup_and_installs_both_project_skills() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    let prefix = "# Existing credentials\nGEMINI_API_KEY=fixture-google-private\n";
    fs::write(root.join(".env"), prefix).unwrap();
    let mut terminal = Terminal::start(root, false);
    terminal.wait("Recommended setup — Google");
    terminal.send("\r");
    terminal.wait("Setup checklist");
    terminal.send("\r"); // Locally ready: Finish is selected without entering a voice ID.
    terminal.wait("Save these reviewed choices?");
    assert_eq!(fs::read_to_string(root.join(".env")).unwrap(), prefix);
    assert!(!root.join("framechord").exists());
    terminal.send("Save\r");
    terminal.success();
    assert_eq!(
        framechord::setup::status(root).unwrap()["setup_complete"],
        true
    );
    assert!(root.join(".agents/skills/framechord/SKILL.md").is_file());
    assert!(root.join(".claude/skills/framechord/SKILL.md").is_file());
    assert!(
        fs::read_to_string(root.join(".env"))
            .unwrap()
            .starts_with(prefix)
    );
    assert!(!terminal.output.contains("fixture-google-private"));
}
#[test]
fn second_welcome_option_selects_openai_explicitly() {
    let t = tempfile::tempdir().unwrap();
    fs::write(
        t.path().join(".env"),
        "OPENAI_API_KEY=fixture-openai-private\n",
    )
    .unwrap();
    let mut terminal = Terminal::start(t.path(), false);
    terminal.wait("Welcome to FrameChord");
    terminal.send("\u{1b}[B\r");
    terminal.wait("Setup checklist");
    terminal.send("\r");
    terminal.wait("Save these reviewed choices?");
    terminal.send("Save\r");
    terminal.success();
    let config = framechord::project::config(t.path()).unwrap();
    assert!(
        config
            .defaults
            .values()
            .all(|s| s.provider == framechord::contracts::Provider::Openai)
    );
    assert!(!terminal.output.contains("fixture-openai-private"));
}
#[test]
fn arrow_setup_completes_with_masked_keys_and_a_voice_after_review() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("terminal project");
    let mut p = Terminal::start(&root, false);
    p.wait("Welcome to FrameChord");
    p.send("\r");
    p.wait("Setup checklist");
    // Exercise an actual arrow selection from Features to API keys.
    p.send("\u{1b}[B\r");
    p.wait("Project credentials");
    p.send("\r");
    p.wait("GEMINI_API_KEY (hidden");
    p.send("fixture-google-hidden\r");
    p.wait("Setup checklist");
    p.send("Voices\r");
    p.wait("Voice bindings");
    p.send("\r");
    p.wait("Voice ID (blank clears)");
    p.send("\u{7f}\u{7f}\u{7f}\u{7f}fixture-voice\r");
    p.wait("Setup checklist");
    assert!(!root.exists(), "navigating setup must not write files");
    p.send("Review and finish\r");
    p.wait("Save these reviewed choices?");
    p.send("Save\r");
    p.success();
    let c = framechord::project::config(&root).unwrap();
    assert_eq!(
        c.default_settings[&framechord::contracts::Media::Speech]
            .voice
            .as_deref(),
        Some("fixture-voice")
    );
    assert!(framechord::setup::status(&root).unwrap()["setup_complete"] == true);
    assert!(!p.output.contains("fixture-google-hidden"));
    assert!(!p.output.contains("\u{1b}[32m"));
}
#[test]
fn escape_returns_to_checklist_and_discard_writes_nothing() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("discard");
    let mut p = Terminal::start(&root, false);
    p.wait("Welcome to FrameChord");
    p.send("\r");
    p.wait("Setup checklist");
    p.send("Features\r");
    p.wait("Choose a feature");
    p.send("\u{1b}");
    p.wait("Setup checklist");
    p.send("Discard changes\r");
    p.wait("Discard all unsaved changes?");
    p.send("Discard and exit\r");
    assert_eq!(p.exit_status().exit_code(), 130);
    assert!(!root.exists());
}
#[test]
fn numbered_menu_saves_a_draft_and_existing_init_resumes_it() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("plain");
    let mut p = Terminal::start(&root, true);
    p.wait("Choice [1]");
    p.send("1\r");
    p.wait("Setup checklist");
    p.wait("Choice [1]");
    p.send("7\r");
    p.wait("Finish is unavailable");
    p.wait("Choice [1]");
    p.send("8\r");
    p.wait("Save these reviewed choices?");
    p.wait("Choice [2]");
    p.send("1\r");
    p.success();
    assert!(root.join("framechord/config.yaml").exists());
    assert_eq!(
        fs::read_to_string(root.join(".env")).unwrap(),
        framechord::credentials::ENV_TEMPLATE
    );
    let before = fs::read(root.join("framechord/config.yaml")).unwrap();
    drop(p);
    let mut resumed = Terminal::start(&root, true);
    resumed.wait("Setup checklist");
    assert!(!resumed.output.contains("Welcome to FrameChord"));
    resumed.wait("Choice [1]");
    resumed.send("9\r");
    resumed.wait("Discard all unsaved changes?");
    resumed.wait("Choice [1]");
    resumed.send("2\r");
    assert_eq!(
        before,
        fs::read(root.join("framechord/config.yaml")).unwrap()
    );
}

#[test]
fn agent_menu_toggles_a_native_target() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    fs::write(root.join(".env"), "GEMINI_API_KEY=fixture-google-private\n").unwrap();
    let mut terminal = Terminal::start(root, false);
    terminal.wait("Recommended setup — Google");
    terminal.send("\r");
    terminal.wait("Setup checklist");
    terminal.send("Agent skills\r");
    terminal.wait("Install project-local skills");
    terminal.send("Choose agents individually\r");
    terminal.wait("Toggle agents");
    terminal.send("GitHub Copilot\r");
    terminal.wait("[x] GitHub Copilot");
    terminal.send("Done\r");
    terminal.wait("Setup checklist");
    terminal.send("\r");
    terminal.wait("Save these reviewed choices?");
    terminal.send("Save\r");
    terminal.success();
    assert!(root.join(".github/skills/framechord/SKILL.md").is_file());
    assert!(root.join(".agents/skills/framechord/SKILL.md").is_file());
    assert!(root.join(".claude/skills/framechord/SKILL.md").is_file());
    assert_eq!(
        framechord::setup::status(root).unwrap()["skills"]["copilot"],
        true
    );
    assert!(!terminal.output.contains("fixture-google-private"));
}

#[test]
fn numbered_agent_menu_preselects_the_current_choice() {
    // The toggle list highlights the first agent that is currently selected (or Done when
    // none is), so the numbered prompt's default follows the choice instead of row 1.
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("plain-agents");
    let mut p = Terminal::start(&root, true);
    p.wait("Choice [1]");
    p.send("1\r");
    p.wait("Setup checklist");
    p.wait("Choice [1]");
    p.send("4\r");
    p.wait("Install project-local skills");
    p.wait("Choice [1]");
    p.send("4\r");
    p.wait("Toggle agents");
    p.wait("[x] Codex");
    p.wait("Choice [1]");
    p.send("1\r");
    p.wait("Toggle agents");
    p.wait("[ ] Codex");
    p.wait("Choice [2]");
    p.send("2\r");
    p.wait("Toggle agents");
    p.wait("[ ] Claude Code");
    p.wait("Choice [7]");
    p.send("6\r");
    p.wait("Toggle agents");
    p.wait("[x] GitHub Copilot");
    p.wait("Choice [6]");
    p.send("7\r");
    p.wait("Setup checklist");
    p.wait("Choice [1]");
    p.send("8\r");
    p.wait("Save these reviewed choices?");
    p.wait("Choice [2]");
    p.send("1\r");
    p.success();
    assert!(root.join(".github/skills/framechord/SKILL.md").is_file());
    assert!(!root.join(".agents").exists());
    assert!(!root.join(".claude").exists());
}
