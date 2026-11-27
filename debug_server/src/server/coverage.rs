use auxtools_protocol::*;

use super::{
	sessions::{ClientId, COVERAGE_PATIENCE},
	Server
};
use crate::coverage;

impl Server {
	pub(super) fn coverage(&mut self) -> Result<(), String> {
		if coverage::running() {
			return Err("another coverage recording is already running".to_owned());
		}
		if !crate::hot_reload::world_has_debug_info() {
			return Err(NO_LINES.to_owned());
		}
		coverage::start();
		self.sessions.covering = self.sessions.requester();
		Ok(())
	}

	pub(super) fn coverage_ended(&mut self) -> Option<ClientId> {
		let owner = self.sessions.covering.take();
		let recorded = coverage::stop();
		let (owner, recorded) = (owner?, recorded?);
		self.tell(owner, Response::CoverageEnded(recorded));
		Some(owner)
	}

	pub fn world_may_end(&mut self) {
		let Some(owner) = self.coverage_ended() else {
			return;
		};
		self.wait_until_sent(owner, COVERAGE_PATIENCE);
		if self.sessions.clients.contains_key(&owner) {
			coverage::start();
			self.sessions.covering = Some(owner);
		}
	}
}

const NO_LINES: &str = "the world has no line numbers because it was compiled without debug information. Add #define DEBUG to the .dme and rebuild";
