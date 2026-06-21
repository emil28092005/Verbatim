use verbatim::ai::AiAction;
use verbatim::ai::GameSession;
use verbatim::ai::ReplayPlayer;

#[test]
fn replay_deterministic() {
    let mut s1 = GameSession::new_seeded(123);
    s1.init();
    s1.set_recording(true);
    s1.perform_action(&AiAction::MoveRight);
    s1.step(10);
    s1.perform_action(&AiAction::Jump);
    s1.step(10);
    let state1 = s1.get_state();

    s1.save_replay("/tmp/verbatim_test_replay.json")
        .expect("save replay");

    let player = ReplayPlayer::load("/tmp/verbatim_test_replay.json").expect("load replay");
    let s2 = player.play();
    let state2 = s2.get_state();

    assert_eq!(state1.tick, state2.tick, "ticks should match");
    if let (Some(p1), Some(p2)) = (&state1.player, &state2.player) {
        assert_eq!(p1.health, p2.health, "player health should match");
        assert!(
            (p1.pos[0] - p2.pos[0]).abs() < 0.01,
            "player x should match: {} vs {}",
            p1.pos[0],
            p2.pos[0]
        );
        assert!(
            (p1.pos[1] - p2.pos[1]).abs() < 0.01,
            "player y should match: {} vs {}",
            p1.pos[1],
            p2.pos[1]
        );
    }
}

#[test]
fn replay_play_until_tick() {
    let mut s = GameSession::new_seeded(999);
    s.init();
    s.set_recording(true);
    s.perform_action(&AiAction::MoveRight);
    s.step(5);
    s.perform_action(&AiAction::MoveLeft);
    s.step(5);
    s.perform_action(&AiAction::Jump);
    s.step(10);
    s.save_replay("/tmp/verbatim_test_replay2.json")
        .expect("save");

    let player = ReplayPlayer::load("/tmp/verbatim_test_replay2.json").expect("load");
    let s_half = player.play_until_tick(5);
    assert_eq!(
        s_half.tick(),
        5,
        "should stop at tick 5, got {}",
        s_half.tick()
    );

    let s_full = player.play();
    assert_eq!(
        s_full.tick(),
        20,
        "full replay should reach tick 20, got {}",
        s_full.tick()
    );
}

#[test]
fn same_seed_same_state() {
    let mut s1 = GameSession::new_seeded(42);
    s1.init();
    s1.step(30);
    let state1 = s1.get_state();

    let mut s2 = GameSession::new_seeded(42);
    s2.init();
    s2.step(30);
    let state2 = s2.get_state();

    assert_eq!(state1.tick, state2.tick);
    if let (Some(p1), Some(p2)) = (&state1.player, &state2.player) {
        assert!(
            (p1.pos[0] - p2.pos[0]).abs() < 0.01,
            "x mismatch: {} vs {}",
            p1.pos[0],
            p2.pos[0]
        );
        assert!(
            (p1.pos[1] - p2.pos[1]).abs() < 0.01,
            "y mismatch: {} vs {}",
            p1.pos[1],
            p2.pos[1]
        );
    }
}

#[test]
fn pipe_protocol_init_and_step() {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new(env!("CARGO_BIN_EXE_verbatim"))
        .arg("--mode")
        .arg("pipe")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to start process");

    let stdin = child.stdin.as_mut().expect("failed to open stdin");
    writeln!(stdin, "{{\"cmd\":\"init\",\"seed\":42}}").expect("write init");
    stdin.flush().expect("flush");

    let mut output = String::new();
    let stdout = child.stdout.as_mut().expect("failed to open stdout");
    use std::io::Read;
    let mut buf = [0u8; 65536];
    let n = stdout.read(&mut buf).expect("read");
    output.push_str(&String::from_utf8_lossy(&buf[..n]));

    let json: serde_json::Value = serde_json::from_str(output.trim()).expect("parse response");
    assert_eq!(json["ok"], true, "init should succeed: {}", output);

    writeln!(stdin, "{{\"cmd\":\"step\",\"n\":10}}").expect("write step");
    stdin.flush().expect("flush");
    let n = stdout.read(&mut buf).expect("read");
    let output2 = String::from_utf8_lossy(&buf[..n]).to_string();
    let json2: serde_json::Value =
        serde_json::from_str(output2.trim()).expect("parse step response");
    assert_eq!(json2["ok"], true);
    assert_eq!(
        json2["state"]["tick"], 10,
        "tick should be 10 after stepping 10"
    );

    writeln!(stdin, "{{\"cmd\":\"quit\"}}").expect("write quit");
    stdin.flush().expect("flush");
    child.wait().expect("wait");
}
