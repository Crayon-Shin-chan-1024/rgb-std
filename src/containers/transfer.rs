// RGB standard library for working with smart contracts on Bitcoin & Lightning
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2019-2024 by
//     Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2019-2024 LNP/BP Standards Association. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::str::FromStr;

use amplify::confinement::{LargeOrdSet, SmallOrdSet, TinyOrdSet};
use amplify::{ByteArray, Bytes32};
use baid58::{Baid58ParseError, Chunking, FromBaid58, ToBaid58, CHUNKING_32};
use commit_verify::{CommitEncode, CommitEngine, CommitId, CommitmentId, DigestExt, Sha256};
use rgb::{Extension, Operation};

use super::Transfer;
use crate::LIB_NAME_RGB_STD;

/// Transfer identifier.
#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[wrapper(Deref, BorrowSlice, Hex, Index, RangeOps)]
#[display(Self::to_baid58_string)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_STD)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct TransferId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl From<Sha256> for TransferId {
    fn from(hasher: Sha256) -> Self { hasher.finish().into() }
}

impl CommitmentId for TransferId {
    const TAG: &'static str = "urn:lnp-bp:rgb:transfer#2024-02-04";
}

impl ToBaid58<32> for TransferId {
    const HRI: &'static str = "transfer";
    const CHUNKING: Option<Chunking> = CHUNKING_32;
    fn to_baid58_payload(&self) -> [u8; 32] { self.to_byte_array() }
    fn to_baid58_string(&self) -> String { self.to_string() }
}
impl FromBaid58<32> for TransferId {}
impl FromStr for TransferId {
    type Err = Baid58ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { Self::from_baid58_chunked_str(s, ':', '#') }
}
#[allow(clippy::wrong_self_convention)] // We need the method that takes self by ref in order to have simpler APIs in iterators
impl TransferId {
    pub fn to_baid58_string(&self) -> String { format!("{::<#.2}", self.to_baid58()) }
    pub fn to_mnemonic(&self) -> String { self.to_baid58().mnemonic() }
}

impl CommitEncode for Transfer {
    type CommitmentId = TransferId;

    fn commit_encode(&self, e: &mut CommitEngine) {
        e.commit_to_serialized(&self.version);
        e.commit_to_serialized(&self.transfer);

        e.commit_to_serialized(&self.contract_id());
        e.commit_to_serialized(&self.genesis.disclose_hash());
        e.commit_to_set(&TinyOrdSet::from_iter_unsafe(
            self.ifaces.values().map(|pair| pair.iimpl.impl_id()),
        ));

        e.commit_to_set(&LargeOrdSet::from_iter_unsafe(
            self.bundles.iter().map(|ab| ab.bundle.disclose_hash()),
        ));
        e.commit_to_set(&LargeOrdSet::from_iter_unsafe(
            self.extensions.iter().map(Extension::disclose_hash),
        ));
        e.commit_to_set(&SmallOrdSet::from_iter_unsafe(self.terminals_disclose()));

        e.commit_to_set(&SmallOrdSet::from_iter_unsafe(self.attachments.keys().copied()));
        e.commit_to_set(&self.supplements);
        e.commit_to_map(&self.asset_tags);
        e.commit_to_map(&self.signatures);
    }
}

impl Transfer {
    #[inline]
    pub fn transfer_id(&self) -> TransferId { self.commit_id() }
}