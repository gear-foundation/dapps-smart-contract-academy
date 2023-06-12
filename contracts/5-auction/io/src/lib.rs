#![no_std]

use gstd::{prelude::*, ActorId};

pub type TamagotchiId = ActorId;
pub type TransactionId = u64;
pub type Bid = u128;
pub type Duration = u64;

#[derive(Encode, Decode)]
pub enum AuctionAction {
    StartAuction {
        tamagotchi_id: TamagotchiId,
        minimum_bid: Bid,
        duration: u64,
    },
    MakeBid {
        bid: Bid,
    },
    SettleAuction,
    MakeReservation,
    CompleteTx(Transaction),
}

#[derive(Encode, Decode)]
pub enum AuctionEvent {
    AuctionStarted,
    BidMade { bid: Bid },
    AuctionSettled,
    ReservationMade,
}

#[derive(Encode, Decode)]
pub enum AuctionError {
    RerunTransaction,
    UnableToChangeOwner,
    UnableToTransferTokens,
    WrongTx,
    WrongParams,
    WrongState,
    WrongDuration,
    WrongBid,
    NoTx,
    WrongReceivedMessage,
    NotOwner,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq)]
pub enum Transaction {
    StartAuction {
        tamagotchi_id: TamagotchiId,
        bid: Bid,
        duration: u64,
    },
    MakeBid {
        transaction_id: TransactionId,
        bidder: ActorId,
        bid: u128,
    },
    SettleAuction {
        transaction_id: TransactionId,
    },
}
