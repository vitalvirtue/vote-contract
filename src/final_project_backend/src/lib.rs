use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUE_SIZE: u32 = 5000;

#[derive(Debug, CandidType, Deserialize)]
enum Choice {
    Approve,
    Reject,
    Pass
}

#[derive(Debug, CandidType, Deserialize)]
enum VoteError {
    AlreadyVoted,
    ProposalIsNotActive,
    NoSuchProposal,
    AccessRejected,
    UpdateError
}

#[derive(Debug, CandidType, Deserialize)]
struct Proposal {
    description: String,
    approve: u32,
    reject: u32,
    pass: u32,
    is_active: bool,
    voted: Vec<candid::Principal>,
    owner: candid::Principal
}

#[derive(Debug, CandidType, Deserialize)]
struct CreateProposal {
    description: String,
    is_active: bool
}

impl Storable {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static PROPOSAL_MAP: RefCell<StableBTreeMap<u64,Proposal,Memory>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))));
}

#[ic_cdk::query]
fn get_proposal(key: u64) -> Option<Proposal> {
    PROPOSAL_MAP.with(|p: &RefCell<BTreeMap>u64, Proposal, _>>| p.borrow().get(&key))
}

#[ic_cdk::query]
fn get_proposal_count() -> u64 {
    PROPOSAL_MAP.with(|p: &RefCell<BTreeMap>u64, Proposal, _>>| p.borrow().len())
}

#[ic_cdk::update]
fn create_proposal(key: u64, proposal: CreateProposal) -> Option<Proposal> {
    let value: Proposal = Proposal {
        description: proposal.description,
        approve: 0u32,
        reject: 0u32,
        pass: 0u32,
        is_active: proposal.is_active,
        voted: vec![],
        owner: ic_cdk::caller(),
    };
    PROPOSAL_MAP.with(|p: &RefCell<BTreeMap>u64, Proposal, _>>| p.borrow_mut().insert(key, value));
}

#[ic_cdk::update]
fn edit_proposal(key: u64, proposal: CreateProposal) -> Result<(), VoteError> {
    PROPOSAL_MAP.with(|map| {
        let mut map = map.borrow_mut();
        if let Some(existing_proposal) = map.get_mut(&key) {
            if existing_proposal.owner == ic_cdk::caller() {
                existing_proposal.description = proposal.description;
                existing_proposal.is_active = proposal.is_active;
                Ok(())
            } else {
                Err(VoteError::AccessRejected)
            }
        } else {
            Err(VoteError::NoSuchProposal)
        }
    })
}

#[ic_cdk::update]
fn end_proposal(key: u64) -> Result<(), VoteError> {
    PROPOSAL_MAP.with(|map| {
        let mut map = map.borrow_mut();
        if let Some(existing_proposal) = map.get_mut(&key) {
            if existing_proposal.owner == ic_cdk::caller() {
                existing_proposal.is_active = false;
                Ok(())
            } else {
                Err(VoteError::AccessRejected)
            }
        } else {
            Err(VoteError::NoSuchProposal)
        }
    })
}

#[ic_cdk::update]
fn vote(key: u64, choice: Choice) -> Result<(), VoteError> {
    PROPOSAL_MAP.with(|map| {
        let mut map = map.borrow_mut();
        if let Some(existing_proposal) = map.get_mut(&key) {
            if existing_proposal.voted.contains(&ic_cdk::caller()) {
                return Err(VoteError::AlreadyVoted);
            }
            match choice {
                Choice::Approve => existing_proposal.approve += 1,
                Choice::Reject => existing_proposal.reject += 1,
                Choice::Pass => existing_proposal.pass += 1,
            }
            existing_proposal.voted.push(ic_cdk::caller());
            Ok(())
        } else {
            Err(VoteError::NoSuchProposal)
        }
    })
}

