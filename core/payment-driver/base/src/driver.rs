/*
    PaymentDriver is a trait to be implemented by each driver so it can be loaded into the bus.
*/

// External crates

// Workspace uses

// Local uses
use crate::model::{
    Ack, GenericError, GetAccountBalance, GetTransactionBalance, Init, PaymentDetails,
    SchedulePayment, VerifyPayment,
};

// Public revealed uses, required to implement this trait
pub use async_trait::async_trait;
pub use bigdecimal::BigDecimal;
pub use ya_client_model::NodeId;
pub use ya_core_model::identity::{event::Event as IdentityEvent, Error as IdentityError};

#[async_trait(?Send)]
pub trait PaymentDriver {
    async fn account_event(
        &self,
        _db: (),
        _caller: String,
        msg: IdentityEvent,
    ) -> Result<(), IdentityError>;

    async fn get_account_balance(
        &self,
        db: (),
        caller: String,
        msg: GetAccountBalance,
    ) -> Result<BigDecimal, GenericError>;

    // used by bus to bind service
    fn get_name(&self) -> String;
    fn get_platform(&self) -> String;

    async fn get_transaction_balance(
        &self,
        db: (),
        caller: String,
        msg: GetTransactionBalance,
    ) -> Result<BigDecimal, GenericError>;

    async fn init(&self, db: (), caller: String, msg: Init) -> Result<Ack, GenericError>;

    async fn schedule_payment(
        &self,
        db: (),
        caller: String,
        msg: SchedulePayment,
    ) -> Result<String, GenericError>;

    async fn verify_payment(
        &self,
        db: (),
        caller: String,
        msg: VerifyPayment,
    ) -> Result<PaymentDetails, GenericError>;
}