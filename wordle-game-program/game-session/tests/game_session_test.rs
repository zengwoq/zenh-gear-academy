use gstd::{prelude::*, ActorId};
use gtest::{Log, Program, System};
use game_session_io::*;

// 定义常量：单词游戏应用程序ID、游戏会话程序ID、用户ID
const WORDLE_ID: u64 = 1;
const GAME_SESSION_ID: u64 = 2;
const USER1: u64 = 10;

fn setup() -> System {
    let sys = System::new();
    sys.init_logger(); 

    let wordle = Program::from_file(&sys, "../target/wasm32-unknown-unknown/debug/wordle.wasm");
    let game_session = Program::from_file(&sys, "../target/wasm32-unknown-unknown/debug/game_session.wasm");

    // 将USER的ActorId和Wordle程序ID关联并发送消息
    let user_id: ActorId = USER1.into();
    let wordle_id: ActorId = WORDLE_ID.into();
    assert!(!wordle.send(user_id, wordle_id).main_failed());
    assert!(!game_session.send(user_id, wordle_id).main_failed());

    return sys; // 返回系统实例
}

// 测试用户赢得游戏的场景
// #[test]
fn test_win() {
    let sys = setup(); // 初始化系统
    let game_session = sys.get_program(GAME_SESSION_ID).unwrap(); // 获取游戏会话程序

    // 用户开始游戏，并依次检查单词（在测试模式下隐藏的单词是 "house"）
    let user1: ActorId = USER1.into();
    assert!(!game_session.send(user1, SessionAction::StartGame).main_failed());
    assert!(!game_session.send(user1, SessionAction::CheckWord { word: "human".to_string() }).main_failed());
    assert!(!game_session.send(user1, SessionAction::CheckWord { word: "house".to_string() }).main_failed());

    // 读取游戏状态并进行断言验证
    let state: State = game_session.read_state(b"").unwrap();
    assert_eq!(state.user_sessions[0].1.result, SessionResult::Win); // 验证用户赢得游戏

}

// 测试用户在超过猜测次数后输掉游戏的场景
// #[test]
fn test_lose_with_too_many_check() {
    let sys: System = setup(); // 初始化系统
    let game_session = sys.get_program(GAME_SESSION_ID).unwrap(); // 获取游戏会话程序

    // 用户开始游戏，并进行多次猜测
    let user: ActorId = USER1.into();
    assert!(!game_session.send(user, SessionAction::StartGame).main_failed());
    assert!(!game_session.send(user, SessionAction::CheckWord { word: "jknkj".to_string() }).main_failed());
    assert!(!game_session.send(user, SessionAction::CheckWord { word: "abcdf".to_string() }).main_failed());
    assert!(!game_session.send(user, SessionAction::CheckWord { word: "hyuiy".to_string() }).main_failed());
    assert!(!game_session.send(user, SessionAction::CheckWord { word: "ppppp".to_string() }).main_failed());
    assert!(!game_session.send(user, SessionAction::CheckWord { word: "lllll".to_string() }).main_failed());
    assert!(!game_session.send(user, SessionAction::CheckWord { word: "ggggg".to_string() }).main_failed());

    // 读取游戏状态并进行断言验证
    let state: State = game_session.read_state(b"").unwrap();
    assert_eq!(state.user_sessions[0].1.check_count, 6); // 验证用户的猜测次数为6
    assert_eq!(state.user_sessions[0].1.result, SessionResult::Lose); // 验证用户未能赢得游戏
}

// 测试用户因超时输掉游戏的场景
#[test]
fn test_lose_with_timeout() {
    let sys: System = setup(); // 初始化系统
    let game_session = sys.get_program(GAME_SESSION_ID).unwrap(); // 获取游戏会话程序

    // 用户1开始游戏，并进行一次猜测（在测试模式下隐藏的单词是 "house"）
    let user: ActorId = USER1.into();
    assert!(!game_session.send(user, SessionAction::StartGame).main_failed());
    assert!(!game_session.send(user, SessionAction::CheckWord { word: "human".to_string() }).main_failed());

    sys.spend_blocks(10); // 模拟区块时间的流逝

    // 读取游戏状态并进行断言验证
    let state: State = game_session.read_state(b"").unwrap();
    assert_eq!(state.user_sessions[0].0, user); // 验证用户ID
    assert_eq!(state.user_sessions[0].1.result, SessionResult::Lose); // 验证用户因超时未能赢得游戏
}
