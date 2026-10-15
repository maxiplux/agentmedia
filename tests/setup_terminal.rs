//! Exercise real terminal detection, raw key navigation and hidden credentials on native OSes.
use portable_pty::{CommandBuilder, PtySize};
use std::{
    fs,
    io::{Read, Write},
    sync::mpsc,
    time::{Duration, Instant},
};
struct Terminal {
    child: Box<dyn portable_pty::Child + Send + Sync>,
    _master: Box<dyn portable_pty::MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    receiver: mpsc::Receiver<Vec<u8>>,
    output: String,
    cursor: usize,
}
impl Terminal {
    fn start(root: &std::path::Path, plain: bool) -> Self {
        let pair = portable_pty::native_pty_system()
            .openpty(PtySize {
                rows: 45,
                cols: 150,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_mediaagent"));
        cmd.args(["--color", "never", "init"]);
        cmd.arg(root);
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
        let writer = pair.master.take_writer().unwrap();
        let (tx, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut buf = [0; 4096];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 || tx.send(buf[..n].to_vec()).is_err() {
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
            if let Some(i) = self.output[self.cursor..].find(token) {
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
        std::thread::sleep(Duration::from_millis(50));
        self.writer.write_all(keys.as_bytes()).unwrap();
        self.writer.flush().unwrap();
    }
    fn success(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(
                    status.success(),
                    "terminal exit {}: {}",
                    status.exit_code(),
                    self.output
                );
                return;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(25));
        }
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
#[test]
fn arrow_setup_completes_with_masked_keys_and_a_voice_after_review() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("terminal project");
    let mut p = Terminal::start(&root, false);
    p.wait("Welcome to MediaAgent");
    p.send("\r");
    p.wait("Setup checklist");
    // Exercise an actual arrow selection from Features to API keys.
    p.send("\u{1b}[B\r");
    p.wait("Project credentials");
    p.send("\r");
    p.wait("OPENAI_API_KEY (hidden");
    p.send("fixture-openai-hidden\r");
    p.wait("Setup checklist");
    p.send("API keys\r");
    p.wait("Project credentials");
    p.send("FISH_API_KEY\r");
    p.wait("FISH_API_KEY (hidden");
    p.send("fixture-fish-hidden\r");
    p.wait("Setup checklist");
    p.send("Voices\r");
    p.wait("Voice bindings");
    p.send("\r");
    p.wait("Voice ID (blank clears)");
    p.send("fixture-voice\r");
    p.wait("Setup checklist");
    assert!(!root.exists(), "navigating setup must not write files");
    p.send("Review and finish\r");
    p.wait("Save these reviewed choices?");
    p.send("Save\r");
    p.success();
    let c = mediaagent::project::config(&root).unwrap();
    assert_eq!(
        c.default_settings[&mediaagent::contracts::Media::Speech]
            .voice
            .as_deref(),
        Some("fixture-voice")
    );
    assert!(mediaagent::setup::status(&root).unwrap()["setup_complete"] == true);
    assert!(!p.output.contains("fixture-openai-hidden"));
    assert!(!p.output.contains("fixture-fish-hidden"));
    assert!(!p.output.contains("\u{1b}[32m"));
}
#[test]
fn escape_returns_to_checklist_and_discard_writes_nothing() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("discard");
    let mut p = Terminal::start(&root, false);
    p.wait("Welcome to MediaAgent");
    p.send("\r");
    p.wait("Setup checklist");
    p.send("Features\r");
    p.wait("Choose a feature");
    p.send("\u{1b}");
    p.wait("Setup checklist");
    p.send("Discard changes\r");
    p.wait("Discard all unsaved changes?");
    p.send("Discard and exit\r");
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(status) = p.child.try_wait().unwrap() {
            assert_eq!(status.exit_code(), 130);
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(25));
    }
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
    assert!(root.join("mediaagent/config.yaml").exists());
    assert_eq!(fs::read_to_string(root.join(".env")).unwrap(), "");
    let before = fs::read(root.join("mediaagent/config.yaml")).unwrap();
    drop(p);
    let mut resumed = Terminal::start(&root, true);
    resumed.wait("Setup checklist");
    assert!(!resumed.output.contains("Welcome to MediaAgent"));
    resumed.wait("Choice [1]");
    resumed.send("9\r");
    resumed.wait("Discard all unsaved changes?");
    resumed.wait("Choice [1]");
    resumed.send("2\r");
    assert_eq!(
        before,
        fs::read(root.join("mediaagent/config.yaml")).unwrap()
    );
}
