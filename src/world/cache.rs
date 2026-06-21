use crate::entity::item::{ItemManager, ItemType};
use crate::entity::player::Player;
use crate::entity::EntityManager;
use crate::world::chunked_grid::ChunkedGrid;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;

const CACHE_VERSION: u32 = 2;

#[derive(Serialize, Deserialize)]
struct CacheMeta {
    version: u32,
    seed: u64,
    player_x: f32,
    player_y: f32,
    depth: u32,
    items: Vec<CachedItem>,
}

#[derive(Serialize, Deserialize)]
struct CachedItem {
    typ: String,
    x: i32,
    y: i32,
}

pub struct WorldCache;

impl WorldCache {
    pub fn path(root: &str, seed: u64) -> PathBuf {
        PathBuf::from(root).join(format!("seed_{}", seed))
    }

    pub fn meta_path(root: &str, seed: u64) -> PathBuf {
        Self::path(root, seed).join("meta.json")
    }

    pub fn chunk_path(root: &str, seed: u64, cx: i32, cy: i32) -> PathBuf {
        Self::path(root, seed).join(format!("chunk_{}_{}.bin", cx, cy))
    }

    pub fn meta_exists(root: &str, seed: u64) -> bool {
        Self::meta_path(root, seed).exists()
    }

    pub fn chunk_exists(root: &str, seed: u64, cx: i32, cy: i32) -> bool {
        Self::chunk_path(root, seed, cx, cy).exists()
    }

    pub fn save_meta(
        root: &str,
        seed: u64,
        player_x: f32,
        player_y: f32,
        depth: u32,
        items: &ItemManager,
    ) -> io::Result<()> {
        let path = Self::path(root, seed);
        std::fs::create_dir_all(&path)?;
        let meta = CacheMeta {
            version: CACHE_VERSION,
            seed,
            player_x,
            player_y,
            depth,
            items: items
                .all()
                .iter()
                .map(|i| CachedItem {
                    typ: i.name().to_string(),
                    x: i.x,
                    y: i.y,
                })
                .collect(),
        };
        let meta_json = serde_json::to_string_pretty(&meta)?;
        std::fs::write(path.join("meta.json"), meta_json)
    }

    pub fn load_meta(
        root: &str,
        seed: u64,
        player: &mut Player,
        entities: &mut EntityManager,
        items: &mut ItemManager,
    ) -> io::Result<u32> {
        let path = Self::path(root, seed);
        let meta_json = std::fs::read_to_string(path.join("meta.json"))?;
        let meta: CacheMeta = serde_json::from_str(&meta_json)
            .map_err(|e| io::Error::other(format!("cache meta parse: {}", e)))?;
        if meta.version != CACHE_VERSION {
            return Err(io::Error::other("cache version mismatch"));
        }
        player.spawn_at(entities, meta.player_x, meta.player_y);
        items.all_mut().clear();
        for ci in meta.items {
            if let Some(typ) = ItemType::from_name(&ci.typ) {
                items.spawn(typ, ci.x, ci.y);
            }
        }
        Ok(meta.depth)
    }

    pub fn save_chunk(
        root: &str,
        seed: u64,
        cx: i32,
        cy: i32,
        grid: &ChunkedGrid,
    ) -> io::Result<()> {
        let path = Self::chunk_path(root, seed, cx, cy);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        grid.save_chunk(path.to_str().unwrap(), cx, cy)
    }

    pub fn load_chunk(
        root: &str,
        seed: u64,
        cx: i32,
        cy: i32,
        grid: &mut ChunkedGrid,
    ) -> io::Result<()> {
        let path = Self::chunk_path(root, seed, cx, cy);
        grid.load_chunk(path.to_str().unwrap(), cx, cy)
    }

    pub fn save_all_loaded(root: &str, seed: u64, grid: &ChunkedGrid) -> io::Result<()> {
        let path = Self::path(root, seed);
        std::fs::create_dir_all(&path)?;
        for (&(cx, cy), chunk) in &grid.chunks {
            if chunk.modified || chunk.was_modified {
                let file = Self::chunk_path(root, seed, cx as i32, cy as i32);
                grid.save_chunk(file.to_str().unwrap(), cx as i32, cy as i32)?;
            }
        }
        Ok(())
    }
}
