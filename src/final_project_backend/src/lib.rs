// External libraries used in Rust are defined here
use candid::{CandidType, Decode, Deserialize, Encode}; // Enables the availability of external libraries
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory}; // Contains necessary structures for memory management
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable}; // Contains structures for storage operations
use std::{borrow::Cow, cell::RefCell}; // Includes certain structures from the standard library

// Defines the type of virtual memory
type Memory = VirtualMemory<DefaultMemoryImpl>;

// Defines a constant value specifying the maximum size of Proposal values
const MAX_VALUE_SIZE: u32 = 5000;

// Defines an enum representing choices users can make
#[derive(Debug, CandidType, Deserialize)]
enum Choice {
    Approve,
    Reject,
    Pass,
}

// Defines an enum representing errors that can occur during voting
#[derive(Debug, CandidType, Deserialize)]
enum VoteError {
    AlreadyVoted,
    ProposalIsNotActive,
    NoSuchProposal,
    AccessRejected,
    UpdateError,
}

// Defines a struct representing a Proposal
#[derive(Debug, CandidType, Deserialize)]
struct Proposal {
    description: String,           // Description field of the Proposal
    approve: u32,                  // Approval count
    reject: u32,                   // Rejection count
    pass: u32,                     // Pass count
    is_active: bool,               // Field indicating if the Proposal is active
    voted: Vec<candid::Principal>, // List of users who voted
    owner: candid::Principal,      // Owner of the Proposal
}

// Defines a struct containing necessary information to create a new Proposal
#[derive(Debug, CandidType, Deserialize)]
struct CreateProposal {
    description: String, // Description field of the Proposal
    is_active: bool,     // Field indicating if the Proposal is active
}

// Implements traits for storable data types
impl Storable for Proposal {
    // Function to convert data to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    // Function to convert bytes to data
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implements traits for bounded storable data types
impl BoundedStorable for Proposal {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE; // Maximum size
    const IS_FIXED_SIZE: bool = false; // Whether it's a fixed size or not
}

// Thread-local memory manager and Proposal map are defined
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static PROPOSAL_MAP: RefCell<StableBTreeMap<u64, Proposal, Memory>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))));
}

// Function defined to get a Proposal
#[ic_cdk::query]
fn get_proposal(key: u64) -> Option<Proposal> {
    PROPOSAL_MAP.with(|p| p.borrow().get(&key))
}

// Function defined to get the count of Proposals
#[ic_cdk::query]
fn get_proposal_count() -> u64 {
    PROPOSAL_MAP.with(|p| p.borrow().len())
}

// Function defined to create a new Proposal
#[ic_cdk::update]
fn create_proposal(key: u64, proposal: CreateProposal) -> Option<Proposal> {
    // Creates a new Proposal and adds it to the Proposal map
    let value: Proposal = Proposal {
        description: proposal.description,
        approve: 0u32,
        reject: 0u32,
        pass: 0u32,
        is_active: proposal.is_active,
        voted: vec![],
        owner: ic_cdk::caller(),
    };
    PROPOSAL_MAP.with(|p| p.borrow_mut().insert(key, value))
}
