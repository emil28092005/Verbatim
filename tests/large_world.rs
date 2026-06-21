use verbatim::game::Game;

#[test]
#[ignore = "slow: generates a 12500x12500 world"]
fn large_world_initialization() {
    let mut game = Game::new_random();
    assert!(game.grid.is_infinite());
    game.init_world();
    let (px, py) = game.player.center(&game.entities);
    assert!(px >= 0.0 && py >= 0.0);
    let foot_x = px as i32;
    let foot_y = (py + 3.0).ceil() as i32;
    assert!(
        !game.grid.get(foot_x, foot_y).is_solid(),
        "player spawn should not be inside solid: px={} py={} foot=({},{}) mat={:?}",
        px,
        py,
        foot_x,
        foot_y,
        game.grid.get(foot_x, foot_y).material
    );
    if let Some(ref root) = game.cache_dir {
        let meta = verbatim::world::cache::WorldCache::meta_path(root, game.seed);
        assert!(
            meta.exists(),
            "world cache should be written after generation"
        );
    }
}

#[test]
#[ignore = "slow: loads cached 12500x12500 world"]
fn large_world_cache_roundtrip() {
    let mut game = Game::new_random();
    game.init_world();
    let (px, py) = game.player.center(&game.entities);
    let item_count = game.items.all().len();
    let root = game.cache_dir.clone().unwrap();
    let seed = game.seed;

    let mut game2 = Game::new_random();
    game2.seed = seed;
    game2.ca.seed(seed);
    game2.cache_dir = Some(root);
    game2.init_world();

    let (px2, py2) = game2.player.center(&game2.entities);
    assert!((px - px2).abs() < 0.01 && (py - py2).abs() < 0.01);
    assert_eq!(game2.items.all().len(), item_count);
    assert!(game2.grid.is_infinite());
}
