#![no_std]

use gmeta::{InOut, Metadata};
use gstd::{prelude::*, ActorId};
use store_io::{AttributeId, TamagotchiId};

pub struct BattleMetadata;

impl Metadata for BattleMetadata {
    type Init = InOut<ActorId, ()>;
    type Handle = InOut<BattleAction, BattleEvent>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = Battle;
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct Battle {
    pub players: Vec<Player>,
    pub state: BattleState,
    pub current_turn: u8,
    pub winner: ActorId,
    pub steps: u8,
    pub tmg_store_id: ActorId,
}

#[derive(Default, Clone, Encode, Decode, TypeInfo)]
pub struct Player {
    pub owner: ActorId,
    pub tmg_id: TamagotchiId,
    pub energy: u16,
    pub power: u16,
    pub attributes: BTreeSet<AttributeId>,
}

#[derive(Default, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum BattleState {
    #[default]
    Registration,
    Moves,
    Waiting,
    GameIsOver,
}

#[derive(Encode, Decode, TypeInfo, Debug)]
pub enum BattleAction {
    Register { tmg_id: TamagotchiId },
    MakeMove,
    UpdateInfo,
    StartNewGame,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum BattleEvent {
    Registered { tmg_id: TamagotchiId },
    MoveMade,
    GoToWaitingState,
    GameIsOver,
    InfoUpdated,
    NewGame,
}
