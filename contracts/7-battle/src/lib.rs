#![no_std]

use battle_io::{BattleAction, BattleEvent, BattleState, Player};
use core::array;
use gstd::{exec, msg, prelude::*, ActorId};
use store_io::*;
use tmg_io::{TmgAction, TmgEvent};
pub const SWORD_ID: AttributeId = 1;
pub const SWORD_POWER: u16 = 5;
const GAS_AMOUNT: u64 = 10_000_000_000;
const TIME_FOR_UPDATE: u32 = 6;
const MAX_STEPS_FOR_ROUND: u8 = 5;
const MAX_POWER: u16 = 10_000;
const MIN_POWER: u16 = 3_000;

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct Battle {
    players: Vec<Player>,
    state: BattleState,
    current_turn: u8,
    winner: ActorId,
    steps: u8,
    tmg_store_id: ActorId,
}

static mut BATTLE: Option<Battle> = None;
impl Battle {
    async fn register(&mut self, tmg_id: &TamagotchiId) {
        assert_eq!(
            self.state,
            BattleState::Registration,
            "The game has already started"
        );

        let attributes = get_attributes(&self.tmg_store_id, tmg_id).await;
        let power = genetate_power();
        let energy = MAX_POWER - power;
        let owner = get_owner(tmg_id).await;
        let player = Player {
            owner,
            tmg_id: *tmg_id,
            energy,
            power,
            attributes,
        };
        self.players.push(player);
        if self.players.len() == 2 {
            self.current_turn = get_turn();
            self.state = BattleState::Moves;
        }
        msg::reply(BattleEvent::Registered { tmg_id: *tmg_id }, 0)
            .expect("Error during a reply `BattleEvent::Registered");
    }
    fn make_move(&mut self) {
        assert_eq!(
            self.state,
            BattleState::Moves,
            "The game is not in `Moves` state"
        );
        let turn = self.current_turn as usize;
        let next_turn = (turn + 1) % 2;
        let player = self.players[turn].clone();
        assert_eq!(
            player.owner,
            msg::source(),
            "You are not in the game or it is not your turn"
        );
        let mut opponent = self.players[next_turn].clone();
        let sword_power = if player.attributes.contains(&SWORD_ID) {
            SWORD_POWER
        } else {
            1
        };

        opponent.energy = opponent
            .energy
            .saturating_sub((sword_power * player.power) / 5);
        self.players[next_turn] = opponent.clone();
        // check if opponent lost
        if opponent.energy == 0 {
            self.state = BattleState::GameIsOver;
            self.winner = player.tmg_id;
            msg::reply(BattleEvent::GameIsOver, 0)
                .expect("Error in sending a reply `BattleEvent::GameIsOver`");
            return;
        }
        if self.steps <= MAX_STEPS_FOR_ROUND {
            self.steps += 1;
            self.current_turn = next_turn as u8;
            msg::reply(BattleEvent::MoveMade, 0)
                .expect("Error in sending a reply `BattleEvent::MoveMade`");
        } else {
            self.state = BattleState::Waiting;
            self.steps = 0;
            msg::send_with_gas_delayed(
                exec::program_id(),
                BattleAction::UpdateInfo,
                GAS_AMOUNT,
                0,
                TIME_FOR_UPDATE,
            )
            .expect("Error in sending a delayed message `BattleAction::UpdateInfo`");
            msg::reply(BattleEvent::GoToWaitingState, 0)
                .expect("Error in sending a reply `BattleEvent::MoveMade`");
        }
    }
    async fn update_info(&mut self) {
        assert_eq!(
            msg::source(),
            exec::program_id(),
            "Only contract itself can call that action"
        );
        assert_eq!(
            self.state,
            BattleState::Waiting,
            "The contract must be in `Waiting` state"
        );

        for i in 0..2 {
            let player = &mut self.players[i];
            let attributes = get_attributes(&self.tmg_store_id, &player.tmg_id).await;
            player.attributes = attributes;
        }
        self.state = BattleState::Moves;
        self.current_turn = get_turn();
        msg::reply(BattleEvent::InfoUpdated, 0)
            .expect("Error during a reply `BattleEvent::InfoUpdated");
    }

    fn start_new_game(&mut self) {
        assert_eq!(
            self.state,
            BattleState::GameIsOver,
            "The previous game must be over"
        );
        self.winner = ActorId::zero();
        self.current_turn = 0;
        self.players = Vec::new();
        self.state = BattleState::Registration;
        msg::reply(BattleEvent::NewGame, 0).expect("Error during a reply `BattleEvent::NewGame");
    }
}

#[gstd::async_main]
async fn main() {
    let action: BattleAction = msg::load().expect("Unable to decode `BattleAction`");
    let battle = unsafe { BATTLE.get_or_insert(Default::default()) };
    match action {
        BattleAction::Register { tmg_id } => battle.register(&tmg_id).await,
        BattleAction::MakeMove => battle.make_move(),
        BattleAction::UpdateInfo => battle.update_info().await,
        BattleAction::StartNewGame => battle.start_new_game(),
    }
}

#[no_mangle]
extern "C" fn init() {
    let tmg_store_id: ActorId = msg::load().expect("Unable to decode `ActorId");
    let battle = Battle {
        tmg_store_id,
        ..Default::default()
    };
    unsafe { BATTLE = Some(battle) };
}

pub async fn get_owner(tmg_id: &ActorId) -> ActorId {
    let reply: TmgEvent = msg::send_for_reply_as(*tmg_id, TmgAction::Owner, 0)
        .expect("Error in sending a message `TmgAction::Owner")
        .await
        .expect("Unable to decode TmgEvent");
    if let TmgEvent::Owner(owner) = reply {
        owner
    } else {
        panic!("Wrong received message");
    }
}

pub fn get_turn() -> u8 {
    let random_input: [u8; 32] = array::from_fn(|i| i as u8 + 1);
    let (random, _) = exec::random(random_input).expect("Error in getting random number");
    random[0] % 2
}

pub fn genetate_power() -> u16 {
    let random_input: [u8; 32] = array::from_fn(|i| i as u8 + 1);
    let (random, _) = exec::random(random_input).expect("Error in getting random number");
    let bytes: [u8; 2] = [random[0], random[1]];
    let random_power: u16 = u16::from_be_bytes(bytes) % MAX_POWER;
    if random_power < MIN_POWER {
        return MAX_POWER / 2;
    }
    random_power
}

async fn get_attributes(tmg_store_id: &ActorId, tmg_id: &TamagotchiId) -> BTreeSet<AttributeId> {
    let reply: StoreEvent = msg::send_for_reply_as(
        *tmg_store_id,
        StoreAction::GetAttributes {
            tamagotchi_id: *tmg_id,
        },
        0,
    )
    .expect("Error in sending a message `StoreAction::GetAttributes")
    .await
    .expect("Unable to decode `StoreEvent`");
    if let StoreEvent::Attributes { attributes } = reply {
        attributes
    } else {
        panic!("Wrong received message");
    }
}

#[no_mangle]
extern "C" fn state() {
    let battle = unsafe { BATTLE.get_or_insert(Default::default()) };
    msg::reply(battle, 0).expect("Failed to share state");
}

#[no_mangle]
extern "C" fn metahash() {
    let metahash: [u8; 32] = include!("../.metahash");
    msg::reply(metahash, 0).expect("Failed to share metahash");
}
