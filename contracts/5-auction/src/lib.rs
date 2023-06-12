#![no_std]

use auction_io::*;
use ft_main_io::{FTokenAction, FTokenEvent, LogicAction};
use gstd::{errors::ContractError, exec, msg, prelude::*, ActorId, ReservationId};
use tmg_io::*;
const MIN_DURATION: u64 = 300_000;
const RESERVATION_AMOUNT: u64 = 50_000_000_000;
const RESERVATION_DURATION: u32 = 86400;
const SYSTEM_GAS: u64 = 1_000_000_000;
#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    ReadyToStart,
    InProcess,
}

impl Default for Status {
    fn default() -> Self {
        Self::ReadyToStart
    }
}

static mut AUCTION: Option<Auction> = None;

#[derive(Default)]
pub struct Auction {
    tamagotchi_id: TamagotchiId,
    status: Status,
    current_bid: u128,
    current_bidder: ActorId,
    ft_contract_id: ActorId,
    transaction: Option<Transaction>,
    transaction_id: TransactionId,
    ended_at: u64,
    prev_tmg_owner: ActorId,
    reservations: Vec<ReservationId>,
}

impl Auction {
    async fn start_auction(
        &mut self,
        tamagotchi_id: &TamagotchiId,
        minimum_bid: Bid,
        duration: u64,
    ) -> Result<AuctionEvent, AuctionError> {
        if self.status != Status::ReadyToStart {
            return Err(AuctionError::WrongState);
        }

        // Check if there is already a pending transaction
        if let Some(tx) = self.transaction.clone() {
            match tx {
                Transaction::StartAuction {
                    tamagotchi_id: prev_tmg_id,
                    bid,
                    duration: prev_duration,
                } => {
                    if *tamagotchi_id != prev_tmg_id
                        || bid != minimum_bid
                        || duration != prev_duration
                    {
                        return Err(AuctionError::WrongParams);
                    }
                    return self.complete_tx(tx).await;
                }
                _ => {
                    return Err(AuctionError::WrongTx);
                }
            }
        }

        if duration < MIN_DURATION {
            return Err(AuctionError::WrongDuration);
        }

        let tx = Transaction::StartAuction {
            tamagotchi_id: *tamagotchi_id,
            bid: minimum_bid,
            duration,
        };
        self.transaction = Some(tx.clone());

        self.complete_tx(tx).await
    }

    async fn make_bid(&mut self, bid: u128) -> Result<AuctionEvent, AuctionError> {
        if self.status != Status::InProcess {
            return Err(AuctionError::WrongState);
        }

        // Check if there is already a pending transaction
        if let Some(tx) = self.transaction.clone() {
            match tx {
                Transaction::MakeBid {
                    transaction_id: _,
                    bidder,
                    bid: prev_bid,
                } => {
                    let result = self.complete_tx(tx).await;
                    if bidder == msg::source() && bid == prev_bid {
                        return result;
                    }
                }
                _ => {
                    return Err(AuctionError::WrongTx);
                }
            }
        }

        if bid <= self.current_bid {
            return Err(AuctionError::WrongBid);
        }

        let transaction_id = self.transaction_id;
        let bidder = msg::source();
        self.transaction_id = self.transaction_id.wrapping_add(2);
        let tx = Transaction::MakeBid {
            transaction_id,
            bidder,
            bid,
        };
        self.transaction = Some(tx.clone());
        self.complete_tx(tx).await
    }

    async fn settle_auction(&mut self) -> Result<AuctionEvent, AuctionError> {
        if self.ended_at < exec::block_timestamp() {
            return Err(AuctionError::WrongState);
        }

        // It is possible that there is a pending transaction `MakeBid`
        if let Some(tx) = self.transaction.clone() {
            match tx {
                Transaction::MakeBid { .. } => {
                    _ = self.complete_tx(tx).await;
                }
                Transaction::SettleAuction { .. } => {
                    return self.complete_tx(tx).await;
                }
                _ => {
                    return Err(AuctionError::WrongTx);
                }
            }
        }

        let transaction_id = self.transaction_id;
        self.transaction_id = self.transaction_id.wrapping_add(1);

        let tx = Transaction::SettleAuction { transaction_id };
        self.transaction = Some(tx.clone());
        self.complete_tx(tx).await
    }

    async fn complete_tx(&mut self, tx: Transaction) -> Result<AuctionEvent, AuctionError> {
        match tx {
            Transaction::StartAuction {
                bid,
                duration,
                tamagotchi_id,
            } => {
                let tmg_owner = if let Ok(tmg_owner) = get_owner(&tamagotchi_id).await {
                    tmg_owner
                } else {
                    self.transaction = None;
                    return Err(AuctionError::WrongReceivedMessage);
                };
                // if tamagotchi owner is already the current contract
                // we just change its state and start the auction
                if tmg_owner == exec::program_id() {
                    self.tamagotchi_id = tamagotchi_id;
                    self.status = Status::InProcess;
                    self.current_bid = bid;
                    self.ended_at = exec::block_timestamp() + duration;
                    self.transaction = None;
                    return Ok(AuctionEvent::AuctionStarted);
                };

                // check that owner starts the auction
                if tmg_owner != msg::source() {
                    return Err(AuctionError::NotOwner);
                }

                if change_owner(&self.tamagotchi_id, &exec::program_id())
                    .await
                    .is_err()
                {
                    self.transaction = None;
                    Err(AuctionError::UnableToChangeOwner)
                } else {
                    self.tamagotchi_id = tamagotchi_id;
                    self.status = Status::InProcess;
                    self.current_bid = bid;
                    self.prev_tmg_owner = tmg_owner;
                    self.ended_at = exec::block_timestamp() + duration;
                    self.transaction = None;
                    msg::send_delayed(
                        exec::program_id(),
                        AuctionAction::SettleAuction,
                        0,
                        duration as u32,
                    )
                    .expect("Error in sending a delayed message `AuctionAction::SettleAuction`");
                    Ok(AuctionEvent::AuctionStarted)
                }
            }
            Transaction::MakeBid {
                transaction_id,
                bidder,
                bid,
            } => {
                if transfer_tokens(
                    transaction_id,
                    &self.ft_contract_id,
                    &bidder,
                    &exec::program_id(),
                    bid,
                )
                .await
                .is_err()
                {
                    self.transaction = None;
                    return Err(AuctionError::UnableToTransferTokens);
                }

                // if it is not the first bet
                // we have to return the tokens to the previous bidder
                // since the tokens are on the auction contract
                // the transaction can fail only due to lack of gas
                // it is necessary to rerun the transaction
                if !self.current_bidder.is_zero()
                    && transfer_tokens(
                        transaction_id + 1,
                        &self.ft_contract_id,
                        &exec::program_id(),
                        &self.current_bidder,
                        self.current_bid,
                    )
                    .await
                    .is_err()
                {
                    return Err(AuctionError::RerunTransaction);
                }

                self.current_bid = bid;
                self.current_bidder = bidder;
                Ok(AuctionEvent::BidMade { bid })
            }
            Transaction::SettleAuction { transaction_id } => {
                let tmg_owner = if let Ok(tmg_owner) = get_owner(&self.tamagotchi_id).await {
                    tmg_owner
                } else {
                    return Err(AuctionError::WrongReceivedMessage);
                };
                if tmg_owner == exec::program_id() {
                    if self.current_bidder.is_zero() {
                        if change_owner(&self.tamagotchi_id, &self.prev_tmg_owner)
                            .await
                            .is_err()
                        {
                            return Err(AuctionError::RerunTransaction);
                        };
                    } else {
                        if transfer_tokens(
                            transaction_id,
                            &self.ft_contract_id,
                            &exec::program_id(),
                            &self.prev_tmg_owner,
                            self.current_bid,
                        )
                        .await
                        .is_err()
                        {
                            return Err(AuctionError::RerunTransaction);
                        };

                        if change_owner(&self.tamagotchi_id, &self.current_bidder)
                            .await
                            .is_err()
                        {
                            return Err(AuctionError::RerunTransaction);
                        };
                    }
                }
                self.transaction = None;
                self.prev_tmg_owner = ActorId::zero();
                self.current_bidder = ActorId::zero();
                self.status = Status::ReadyToStart;
                self.ended_at = 0;

                Ok(AuctionEvent::AuctionSettled)
            }
        }
    }

    fn make_reservation(&mut self) -> Result<AuctionEvent, AuctionError> {
        let reservation_id = ReservationId::reserve(RESERVATION_AMOUNT, RESERVATION_DURATION)
            .expect("reservation across executions");
        self.reservations.push(reservation_id);
        Ok(AuctionEvent::ReservationMade)
    }
}

#[gstd::async_main]
async fn main() {
    let action: AuctionAction = msg::load().expect("Unable to decode `AuctionAction`");
    let auction = unsafe { AUCTION.get_or_insert(Default::default()) };
    let reply = match action {
        AuctionAction::StartAuction {
            tamagotchi_id,
            minimum_bid,
            duration,
        } => {
            system_reserve_gas();
            auction
                .start_auction(&tamagotchi_id, minimum_bid, duration)
                .await
        }
        AuctionAction::MakeBid { bid } => {
            system_reserve_gas();
            auction.make_bid(bid).await
        }
        AuctionAction::SettleAuction => {
            system_reserve_gas();
            auction.settle_auction().await
        }
        AuctionAction::MakeReservation => auction.make_reservation(),
        AuctionAction::CompleteTx(tx) => {
            if let Some(_tx) = &auction.transaction {
                if tx == _tx.clone() {
                    auction.complete_tx(tx).await
                } else {
                    Err(AuctionError::WrongTx)
                }
            } else {
                Err(AuctionError::NoTx)
            }
        }
    };
    msg::reply(reply, 0).expect("Failed to encode or reply with `Result<MarketEvent, MarketErr>`");
}

fn system_reserve_gas() {
    exec::system_reserve_gas(SYSTEM_GAS).expect("Error during system gas reservation");
}
async fn transfer_tokens(
    transaction_id: TransactionId,
    token_address: &ActorId,
    from: &ActorId,
    to: &ActorId,
    amount_tokens: u128,
) -> Result<(), ()> {
    exec::system_reserve_gas(2_000_000_000).expect("Error during system gas reservation");
    let result = msg::send_for_reply_as::<_, FTokenEvent>(
        *token_address,
        FTokenAction::Message {
            transaction_id,
            payload: LogicAction::Transfer {
                sender: *from,
                recipient: *to,
                amount: amount_tokens,
            },
        },
        0,
    );

    let (_msg_id, msg_future) = if let Ok(msg_future) = result {
        (msg_future.waiting_reply_to, msg_future)
    } else {
        return Err(());
    };

    let reply = msg_future.await;
    match reply {
        Ok(FTokenEvent::Ok) => Ok(()),
        _ => Err(()),
    }
}

async fn get_owner(tamagotchi_id: &TamagotchiId) -> Result<ActorId, AuctionError> {
    let reply = msg::send_for_reply_as(*tamagotchi_id, TmgAction::Owner, 0)
        .expect("Error in sending a message `TmgAction::Owner` to Tamagotchi contract")
        .await;
    match reply {
        Ok(TmgEvent::Owner(tmg_owner)) => Ok(tmg_owner),
        _ => Err(AuctionError::WrongReceivedMessage),
    }
}

async fn change_owner(
    tamagotchi_id: &TamagotchiId,
    new_owner: &ActorId,
) -> Result<TmgEvent, ContractError> {
    msg::send_for_reply_as::<_, TmgEvent>(*tamagotchi_id, TmgAction::Transfer(*new_owner), 0)
        .expect("Error in sending a message `TmgAction::ChangeOwner` to Tamagotchi contract")
        .await
}

#[no_mangle]
extern "C" fn my_handle_signal() {
    let auction = unsafe { AUCTION.get_or_insert(Default::default()) };
    if let Some(tx) = &auction.transaction {
        let reservation_id = if !auction.reservations.is_empty() {
            auction.reservations.remove(0)
        } else {
            return;
        };
        msg::send_from_reservation(
            reservation_id,
            exec::program_id(),
            AuctionAction::CompleteTx(tx.clone()),
            0,
        )
        .expect("Failed to send message");
    }
}
