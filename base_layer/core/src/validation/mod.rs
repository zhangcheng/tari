// Copyright 2019. The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! The validation module defines the [Validation] trait which describes all code that can perform block,
//! transaction, or other validation tasks. Validators implement the [Validation] trait and can be chained together
//! in a [ValidationPipeline] object to carry out complex validation routines.
//!
//! This module also defines a mock [MockValidator] that is useful for testing components that require validation
//! without having to bring in all sorts of blockchain and communications paraphernalia.

mod error;
pub use error::ValidationError;

pub(crate) mod helpers;

mod traits;
pub use traits::{
    BlockSyncBodyValidation,
    FinalHorizonStateValidation,
    HeaderValidation,
    MempoolTransactionValidation,
    OrphanValidation,
    PostOrphanBodyValidation,
};

pub mod block_validators;
mod difficulty_calculator;
pub use difficulty_calculator::*;
pub mod header_validator;
pub mod mocks;
pub mod transaction_validators;
// pub mod header_validator;

mod chain_balance;
pub use chain_balance::ChainBalanceValidator;

mod header_iter;

#[cfg(test)]
mod test;
