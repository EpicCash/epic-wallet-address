// Copyright 2019 The vault713 Developers
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

pub mod ser;
pub mod v2;
pub mod v3;
use super::Slate;
use crate::wallet::error::ErrorKind;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use v2::SlateV2;
use v3::SlateV3;

pub const CURRENT_SLATE_VERSION: u16 = 2;
pub const EPIC_BLOCK_HEADER_VERSION: u16 = 6;

/// Existing versions of the slate
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum SlateVersion {
	/// V3 (most current)
	V3,
	/// V2 (2.0.0 - Onwards)
	V2,
}

impl Default for SlateVersion {
	fn default() -> Self {
		SlateVersion::try_from(CURRENT_SLATE_VERSION).unwrap()
	}
}

impl TryFrom<u16> for SlateVersion {
	type Error = ErrorKind;

	fn try_from(value: u16) -> Result<Self, Self::Error> {
		match value {
			2 => Ok(SlateVersion::V2),
			v => Err(ErrorKind::SlateVersion(v)),
		}
	}
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
/// Versions are ordered newest to oldest so serde attempts to
/// deserialize newer versions first, then falls back to older versions.
pub enum VersionedSlate {
	/// Current (3.0.0 Onwards )
	V3(SlateV3),
	/// V2 (2.0.0 - Onwards)
	V2(SlateV2),
}

impl VersionedSlate {
	/// Return slate version
	pub fn version(&self) -> SlateVersion {
		match *self {
			VersionedSlate::V3(_) => SlateVersion::V3,
			VersionedSlate::V2(_) => SlateVersion::V2,
		}
	}

	/// convert this slate type to a specified older version
	pub fn into_version(slate: Slate, version: SlateVersion) -> VersionedSlate {
		match version {
			SlateVersion::V3 => VersionedSlate::V3(slate.into()),
			// Left here as a reminder of what needs to be inserted on
			// the release of a new slate
			SlateVersion::V2 => {
				let s = SlateV3::from(slate);
				let s = SlateV2::from(&s);
				VersionedSlate::V2(s)
			}
		}
	}
}

impl From<VersionedSlate> for Slate {
	fn from(slate: VersionedSlate) -> Slate {
		match slate {
			VersionedSlate::V3(s) => {
				let s = SlateV3::from(s);
				Slate::from(s)
			}
			VersionedSlate::V2(s) => {
				let s = SlateV3::from(s);
				Slate::from(s)
			}
		}
	}
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CompatKernelFeatures {
	Plain,
	Coinbase,
	HeightLocked,
}
