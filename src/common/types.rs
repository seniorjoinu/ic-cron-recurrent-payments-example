use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cron::types::{SchedulingInterval, TaskId};

#[derive(Debug)]
pub enum Error {
    InsufficientBalance,
    ZeroQuantity,
    AccessDenied,
    ForbiddenOperation,
}

pub type Controllers = Vec<Principal>;

#[derive(CandidType, Deserialize, Debug)]
pub enum CronTaskKind {
    RecurrentTransfer(RecurrentTransferTask),
    RecurrentMint(RecurrentMintTask),
}

#[derive(CandidType, Deserialize, Debug)]
pub struct RecurrentTransferTask {
    pub from: Principal,
    pub to: Principal,
    pub qty: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct RecurrentMintTask {
    pub to: Principal,
    pub qty: u64,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(CandidType, Deserialize)]
pub struct RecurrentTransferTaskExt {
    pub task_id: TaskId,
    pub from: Principal,
    pub to: Principal,
    pub qty: u64,
    pub scheduled_at: u64,
    pub rescheduled_at: Option<u64>,
    pub scheduling_interval: SchedulingInterval,
}

#[derive(CandidType, Deserialize)]
pub struct RecurrentMintTaskExt {
    pub task_id: TaskId,
    pub to: Principal,
    pub qty: u64,
    pub scheduled_at: u64,
    pub rescheduled_at: Option<u64>,
    pub scheduling_interval: SchedulingInterval,
}
