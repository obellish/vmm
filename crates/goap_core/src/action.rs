use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Action {
	pub duration: i32,
	pub deltas: HashMap<String, i32>,
}
