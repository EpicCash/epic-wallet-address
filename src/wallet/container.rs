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

use super::ErrorKind;
use crate::api::listener::{Listener, ListenerInterface};
use crate::common::config::Wallet713Config;
use crate::common::{Arc, Keychain, Mutex};
use crate::contacts::AddressBook;
use crate::wallet::backend::Backend;
use crate::wallet::types::{HTTPNodeClient, NodeClient, WalletBackend};
use failure::Error;
use epic_keychain::ExtKeychain;
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct Container<W, C, K>
where
	W: WalletBackend<C, K>,
	C: NodeClient,
	K: Keychain,
{
	pub config: Wallet713Config,
	backend: W,
	pub address_book: AddressBook,
	pub account: String,
	pub listeners: HashMap<ListenerInterface, Box<dyn Listener>>,
	phantom_c: PhantomData<C>,
	phantom_k: PhantomData<K>,
}

impl<W, C, K> Container<W, C, K>
where
	W: WalletBackend<C, K>,
	C: NodeClient,
	K: Keychain,
{
	pub fn new(config: Wallet713Config, backend: W, address_book: AddressBook) -> Arc<Mutex<Self>> {
		let container = Self {
			config,
			backend,
			address_book,
			account: String::from("default"),
			listeners: HashMap::with_capacity(4),
			phantom_c: PhantomData,
			phantom_k: PhantomData,
		};
		Arc::new(Mutex::new(container))
	}

	pub fn raw_backend(&mut self) -> &mut W {
		&mut self.backend
	}

	pub fn backend(&mut self) -> Result<&mut W, Error> {
		if !self.backend.connected()? {
			return Err(ErrorKind::NoBackend.into());
		}
		Ok(&mut self.backend)
	}

	pub fn listener(&self, interface: ListenerInterface) -> Result<&Box<dyn Listener>, ErrorKind> {
		self.listeners
			.get(&interface)
			.ok_or(ErrorKind::NoListener(format!("{}", interface)))
	}
}

pub fn create_container(
	config: Wallet713Config,
	address_book: AddressBook,
) -> Result<
	Arc<Mutex<Container<Backend<HTTPNodeClient, ExtKeychain>, HTTPNodeClient, ExtKeychain>>>,
	Error,
> {
	let wallet_config = config.as_wallet_config()?;
	let client = HTTPNodeClient::new(
		&wallet_config.check_node_api_http_addr,
		config.epic_node_secret().clone(),
	);
	let backend = Backend::new(&wallet_config, client)?;
	Ok(Container::new(config, backend, address_book))
}
