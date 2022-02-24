use std::collections::{HashMap, HashSet};

use ic_cdk::export::candid::export_service;
use ic_cdk::export::Principal;
use ic_cdk::{caller, trap};
use ic_cdk_macros::{heartbeat, init, query, update};
use ic_cron::implement_cron;
use ic_cron::types::{Iterations, SchedulingInterval, TaskId};

use crate::common::currency_token::CurrencyToken;
use crate::common::guards::controller_guard;
use crate::common::types::{
    CronTaskKind, RecurrentMintTask, RecurrentMintTaskExt, RecurrentTransferTask,
    RecurrentTransferTaskExt, TokenInfo,
};

mod common;

// ----------------- MAIN LOGIC ------------------

#[init]
fn init(controller: Principal, info: TokenInfo) {
    let token = CurrencyToken {
        balances: HashMap::new(),
        total_supply: 0,
        info,
        controllers: vec![controller],
        recurrent_mint_tasks: HashSet::new(),
        recurrent_transfer_tasks: HashMap::new(),
    };

    unsafe {
        STATE = Some(token);
    }
}

#[update(guard = "controller_guard")]
fn mint(to: Principal, qty: u64, scheduling_interval: Option<SchedulingInterval>) {
    match scheduling_interval {
        Some(interval) => {
            let task_id = cron_enqueue(
                CronTaskKind::RecurrentMint(RecurrentMintTask { to, qty }),
                interval,
            )
            .expect("Mint scheduling failed");

            get_token().register_recurrent_mint_task(task_id);
        }
        None => {
            get_token().mint(to, qty).expect("Minting failed");
        }
    }
}

#[update]
fn transfer(to: Principal, qty: u64, scheduling_interval: Option<SchedulingInterval>) {
    let from = caller();

    match scheduling_interval {
        Some(interval) => {
            let task_id = cron_enqueue(
                CronTaskKind::RecurrentTransfer(RecurrentTransferTask { from, to, qty }),
                interval,
            )
            .expect("Transfer scheduling failed");

            get_token().register_recurrent_transfer_task(from, task_id);
        }
        None => {
            get_token()
                .transfer(from, to, qty)
                .expect("Transfer failed");
        }
    }
}

#[update]
fn burn(qty: u64) {
    get_token().burn(caller(), qty).expect("Burning failed");
}

#[query]
fn get_balance_of(account_owner: Principal) -> u64 {
    get_token().balance_of(&account_owner)
}

#[query]
fn get_total_supply() -> u64 {
    get_token().total_supply
}

#[query]
fn get_info() -> TokenInfo {
    get_token().info.clone()
}

// --------------- RECURRENCE ------------------

implement_cron!();

#[heartbeat]
pub fn tick() {
    let token = get_token();

    for task in cron_ready_tasks() {
        let kind: CronTaskKind = task.get_payload().expect("Unable to decode task payload");

        match kind {
            CronTaskKind::RecurrentMint(mint_task) => {
                token
                    .mint(mint_task.to, mint_task.qty)
                    .expect("Unable to perform scheduled mint");

                if let Iterations::Exact(n) = task.scheduling_interval.iterations {
                    if n == 1 {
                        token.unregister_recurrent_mint_task(task.id);
                    }
                };
            }
            CronTaskKind::RecurrentTransfer(transfer_task) => {
                token
                    .transfer(transfer_task.from, transfer_task.to, transfer_task.qty)
                    .expect("Unable to perform scheduled transfer");

                if let Iterations::Exact(n) = task.scheduling_interval.iterations {
                    if n == 1 {
                        token.unregister_recurrent_transfer_task(transfer_task.from, task.id);
                    }
                };
            }
        }
    }
}

#[update]
pub fn cancel_recurrent_mint_task(task_id: TaskId) -> bool {
    cron_dequeue(task_id).expect("Task id not found");
    get_token().unregister_recurrent_mint_task(task_id)
}

#[query(guard = "controller_guard")]
pub fn get_recurrent_mint_tasks() -> Vec<RecurrentMintTaskExt> {
    get_token()
        .get_recurrent_mint_tasks()
        .into_iter()
        .map(|task_id| {
            let task = get_cron_state().get_task_by_id(&task_id).unwrap();
            let kind: CronTaskKind = task
                .get_payload()
                .expect("Unable to decode a recurrent mint task");

            match kind {
                CronTaskKind::RecurrentTransfer(_) => trap("Invalid task kind"),
                CronTaskKind::RecurrentMint(mint_task) => RecurrentMintTaskExt {
                    task_id: task.id,
                    to: mint_task.to,
                    qty: mint_task.qty,
                    scheduled_at: task.scheduled_at,
                    rescheduled_at: task.rescheduled_at,
                    scheduling_interval: task.scheduling_interval,
                },
            }
        })
        .collect()
}

#[update]
pub fn cancel_my_recurrent_transfer_task(task_id: TaskId) -> bool {
    cron_dequeue(task_id).expect("Task id not found");
    get_token().unregister_recurrent_transfer_task(caller(), task_id)
}

#[query]
pub fn get_my_recurrent_transfer_tasks() -> Vec<RecurrentTransferTaskExt> {
    get_token()
        .get_recurrent_transfer_tasks(caller())
        .into_iter()
        .map(|task_id| {
            let task = get_cron_state().get_task_by_id(&task_id).unwrap();
            let kind: CronTaskKind = task
                .get_payload()
                .expect("Unable to decode a recurrent transfer task");

            match kind {
                CronTaskKind::RecurrentMint(_) => trap("Invalid task kind"),
                CronTaskKind::RecurrentTransfer(transfer_task) => RecurrentTransferTaskExt {
                    task_id: task.id,
                    from: transfer_task.from,
                    to: transfer_task.to,
                    qty: transfer_task.qty,
                    scheduled_at: task.scheduled_at,
                    rescheduled_at: task.rescheduled_at,
                    scheduling_interval: task.scheduling_interval,
                },
            }
        })
        .collect()
}

// ------------------ STATE ----------------------

export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}

static mut STATE: Option<CurrencyToken> = None;

pub fn get_token() -> &'static mut CurrencyToken {
    unsafe { STATE.as_mut().unwrap() }
}
