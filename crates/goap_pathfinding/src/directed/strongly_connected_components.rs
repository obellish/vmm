use std::{
	collections::{HashMap, HashSet},
	hash::Hash,
};

struct Params<N, FN>
where
	N: Eq + Hash,
{
	preorders: HashMap<N, Option<usize>>,
	c: usize,
	successors: FN,
	p: Vec<N>,
	s: Vec<N>,
	scc: Vec<Vec<N>>,
	scca: HashSet<N>,
}

impl<N, FN, IN> Params<N, FN>
where
	N: Clone + Hash + Eq,
	FN: FnMut(&N) -> IN,
	IN: IntoIterator<Item = N>,
{
	fn new(nodes: &[N], successors: FN) -> Self {
		Self {
			preorders: nodes.iter().map(|n| (n.clone(), None)).collect(),
			c: 0,
			successors,
			p: Vec::new(),
			s: Vec::new(),
			scc: Vec::new(),
			scca: HashSet::new(),
		}
	}

	fn recurse_onto(&mut self, v: &N) {
		self.preorders.insert(v.clone(), Some(self.c));
		self.c += 1;
		self.s.push(v.clone());
		self.p.push(v.clone());
		for w in (self.successors)(v) {
			if !self.scca.contains(&w) {
				if let Some(pw) = self.preorders.get(&w).and_then(|w| *w) {
					while self.preorders[&self.p[self.p.len() - 1]].unwrap() > pw {
						self.p.pop();
					}
				} else {
					self.recurse_onto(&w);
				}
			}
		}

		if self.p[self.p.len() - 1] == *v {
			self.p.pop();
			let mut components = Vec::new();
			while let Some(node) = self.s.pop() {
				components.push(node.clone());
				self.scca.insert(node.clone());
				self.preorders.remove(&node);
				if node == *v {
					break;
				}
			}

			self.scc.push(components);
		}
	}
}

pub fn strongly_connected_components_from<N, IN>(
	start: &N,
	successors: impl FnMut(&N) -> IN,
) -> Vec<Vec<N>>
where
	N: Clone + Eq + Hash,
	IN: IntoIterator<Item = N>,
{
	let mut params = Params::new(&[], successors);
	params.recurse_onto(start);
	params.scc
}

pub fn strongly_connected_component<N, IN>(node: &N, successors: impl FnMut(&N) -> IN) -> Vec<N>
where
	N: Clone + Eq + Hash,
	IN: IntoIterator<Item = N>,
{
	strongly_connected_components_from(node, successors)
		.pop()
		.unwrap()
}

pub fn strongly_connected_components<N, IN>(
	nodes: &[N],
	successors: impl FnMut(&N) -> IN,
) -> Vec<Vec<N>>
where
	N: Clone + Eq + Hash,
	IN: IntoIterator<Item = N>,
{
	let mut params = Params::new(nodes, successors);
	while let Some(node) = params.preorders.keys().find(|_| true).cloned() {
		params.recurse_onto(&node);
	}

	params.scc
}
