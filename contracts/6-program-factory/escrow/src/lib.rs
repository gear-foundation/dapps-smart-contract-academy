#![no_std]

use escrow_new_io::*;
use gstd::{msg, prelude::*, ActorId};

#[derive(Debug, PartialEq, Eq)]
enum EscrowState {
    AwaitingPayment,
    AwaitingDelivery,
    Closed,
}

impl Default for EscrowState {
    fn default() -> Self {
        Self::AwaitingPayment
    }
}

#[derive(Default)]
struct Escrow {
    factory_id: ActorId,
    seller: ActorId,
    buyer: ActorId,
    price: u128,
    state: EscrowState,
}

static mut ESCROW: Option<Escrow> = None;

impl Escrow {
    fn deposit(&mut self, account: &ActorId) {
        assert_eq!(
            self.state,
            EscrowState::AwaitingPayment,
            "State must be `AwaitingPayment"
        );

        assert_eq!(
            msg::source(),
            self.factory_id,
            "The message sender must be a factory contract"
        );

        assert_eq!(
            account, &self.buyer,
            "The indicated account must be a buyer"
        );

        assert_eq!(
            msg::value(),
            self.price,
            "The attached value must be equal to set price"
        );

        self.state = EscrowState::AwaitingDelivery;
        msg::reply(EscrowEvent::FundsDeposited, 0)
            .expect("Error in reply `EscrowEvent::FundsDeposited");
    }
    fn confirm_delivery(&mut self, account: &ActorId) {
        assert_eq!(
            self.state,
            EscrowState::AwaitingDelivery,
            "State must be `AwaitingDelivery"
        );

        assert_eq!(
            msg::source(),
            self.factory_id,
            "The message sender must be a factory contract"
        );

        assert_eq!(
            account, &self.buyer,
            "The indicated account must be a buyer"
        );
        self.state = EscrowState::Closed;
        msg::send_with_gas(self.seller, EscrowEvent::PaymentToSeller, 0, self.price)
            .expect("Error in sending funds to the seller");
        msg::reply(EscrowEvent::DeliveryConfirmed, 0).expect("Error during a reply `FactoryEvent`");
    }
}
#[no_mangle]
unsafe extern "C" fn handle() {
    let action: EscrowAction = msg::load().expect("Unable to decode `EscrowAction`");
    let escrow: &mut Escrow = ESCROW.get_or_insert(Default::default());
    match action {
        EscrowAction::Deposit(account) => escrow.deposit(&account),
        EscrowAction::ConfirmDelivery(account) => escrow.confirm_delivery(&account),
    }
}

#[no_mangle]
unsafe extern "C" fn init() {
    let init_config: InitEscrow = msg::load().expect("Error in decoding `InitEscrow`");
    let escrow = Escrow {
        factory_id: msg::source(),
        seller: init_config.seller,
        buyer: init_config.buyer,
        price: init_config.price,
        state: EscrowState::AwaitingPayment,
    };
    ESCROW = Some(escrow);
    msg::reply(EscrowEvent::ProgramInitialized, 0)
        .expect("Error during a reply `EscrowEvent::ProgramInitialized`");
}
