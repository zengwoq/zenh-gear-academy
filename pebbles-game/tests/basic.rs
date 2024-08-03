use gtest::{Log, Program, System};
use pebbles_game_io::*;

const PLAYER: u64 = 100;

fn init_game(sys: &System, difficulty: DifficultyLevel, pebbles_count: u32, max_pebbles_per_turn: u32) -> Program<'_> {
    sys.init_logger();
    let game = Program::current_opt(sys);

    let pebbles_init = PebblesInit {
        difficulty,
        pebbles_count,
        max_pebbles_per_turn,
    };
    let res = game.send(PLAYER, pebbles_init);
    assert!(!res.main_failed());
    game
}

#[test]
fn game_flow() {
    let sys = System::new();
    let game = init_game(&sys, DifficultyLevel::Easy, 15, 2);

    // 检查初始状态
    let state: GameState = game.read_state(b"").unwrap();
    assert_eq!(state.pebbles_count, 15);
    assert_eq!(state.max_pebbles_per_turn, 2);
    assert!(state.pebbles_remaining <= 15 && state.pebbles_remaining >= 13);
    assert!(matches!(state.difficulty, DifficultyLevel::Easy));
    assert!(state.winner.is_none());

    // 玩家回合
    let res = game.send(PLAYER, PebblesAction::Turn(1));
    assert!(!res.main_failed());
    let expected_counter = Log::builder().payload(PebblesEvent::CounterTurn(1));
    assert!(res.contains(&expected_counter) || res.contains(&Log::builder().payload(PebblesEvent::Won(Player::Program))));

    // 检查游戏是否结束
    let state: GameState = game.read_state(b"").unwrap();
    if state.winner.is_some() {
        assert!(matches!(state.winner, Some(Player::Program)));
    } else {
        // 继续游戏直到结束
        loop {
            let state: GameState = game.read_state(b"").unwrap();
            if state.winner.is_some() {
                break;
            }
            let pebbles_to_remove = std::cmp::min(state.pebbles_remaining, state.max_pebbles_per_turn);
            let res = game.send(PLAYER, PebblesAction::Turn(pebbles_to_remove));
            assert!(!res.main_failed());
        }
    }

    // 检查最终状态
    let final_state: GameState = game.read_state(b"").unwrap();
    assert!(final_state.winner.is_some());
    assert_eq!(final_state.pebbles_remaining, 0);
}

#[test]
fn difficulty_levels() {
    let sys = System::new();

    // 简单模式
    let easy_game = init_game(&sys, DifficultyLevel::Easy, 15, 2);
    let easy_state: GameState = easy_game.read_state(b"").unwrap();
    assert!(matches!(easy_state.difficulty, DifficultyLevel::Easy));

    // 困难模式
    let hard_game = init_game(&sys, DifficultyLevel::Hard, 15, 2);
    let hard_state: GameState = hard_game.read_state(b"").unwrap();
    assert!(matches!(hard_state.difficulty, DifficultyLevel::Hard));
}

#[test]
fn invalid_inputs() {
    let sys = System::new();
    let game = Program::current_opt(&sys);

    // 无效的初始化参数
    let invalid_init = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 0,
        max_pebbles_per_turn: 0,
    };
    let res = game.send(PLAYER, invalid_init);
    assert!(res.main_failed());

    // 有效初始化
    let game = init_game(&sys, DifficultyLevel::Easy, 15, 2);

    // 无效的回合操作
    let res = game.send(PLAYER, PebblesAction::Turn(3));
    assert!(res.main_failed());
}

#[test]
fn give_up_and_restart() {
    let sys = System::new();
    let game = init_game(&sys, DifficultyLevel::Easy, 15, 2);

    // 玩家投降
    let res = game.send(PLAYER, PebblesAction::GiveUp);
    assert!(!res.main_failed());

    let state: GameState = game.read_state(b"").unwrap();
    assert!(matches!(state.winner, Some(Player::Program)));

    // 重新开始游戏
    let res = game.send(PLAYER, PebblesAction::Restart {
        difficulty: DifficultyLevel::Hard,
        pebbles_count: 20,
        max_pebbles_per_turn: 3,
    });
    assert!(!res.main_failed());

    let new_state: GameState = game.read_state(b"").unwrap();
    assert_eq!(new_state.pebbles_count, 20);
    assert_eq!(new_state.max_pebbles_per_turn, 3);
    assert!(matches!(new_state.difficulty, DifficultyLevel::Hard));
    assert!(new_state.winner.is_none());
}
