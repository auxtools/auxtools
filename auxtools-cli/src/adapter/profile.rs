//! Start profiling or return an HTML report through `auxtoolsProfile`.

use super::*;
use crate::profile;

/// Arguments for `auxtoolsProfile`
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProfileArguments {
	/// `start` or `stop`
	action: String,
	/// Procs to measure, as `/PATH[#N]`. Every proc if there are none.
	#[serde(default)]
	procs: Vec<String>,
	/// Number of procs to include with instruction listings, by self time.
	#[serde(default = "listed")]
	top: usize
}

const fn listed() -> usize {
	40
}

impl Adapter {
	pub(super) fn profile(&mut self, args: ProfileArguments) -> Handled {
		match args.action.as_str() {
			"start" => {
				let procs = args.procs.iter().map(|proc| named_proc(proc)).collect();
				match self.request(&AuxRequest::Profile(aux::ProfileRequest::Start { procs }))? {
					AuxResponse::Profile(Ok(_)) => Ok(Value::Null),
					AuxResponse::Profile(Err(e)) => Err(e),
					other => Err(unexpected(other))
				}
			}
			"stop" => {
				let report = profile::stop("stopped", Some(args.top), |request| self.request(&request))?;
				Ok(serde_json::json!({
					"seconds": report.seconds,
					"instructions": report.hits,
					"procs": report.procs.len(),
					"html": profile::html(&report)
				}))
			}
			other => Err(format!("unknown profile action '{}': expected start or stop", other))
		}
	}
}
