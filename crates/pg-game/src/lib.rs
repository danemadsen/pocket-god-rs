//! Pocket God's game code: `PocketGodViewController`, the managers, the personas and every
//! logic class, ported from the decompiled original (`tmp/re/decomp/game/`) onto `pg-engine`.
//!
//! Every Objective-C class is a module with its ivar component(s), its [`Class`] constant
//! and one function per method; find any original method by grepping for its selector in the
//! doc comments, e.g. `grep -rn '\[PygmyLogic fall:' crates/pg-game/src`.

pub mod library_logic;
pub mod logic_table;
pub mod prelude;
pub mod send;
pub mod singletons;
pub mod types;

pub mod controller;
pub mod logic;
pub mod managers;
pub mod persona;
pub mod pygmy;
pub mod pygmy_extras;
