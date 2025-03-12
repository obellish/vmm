use std::{
	collections::{HashMap, HashSet, VecDeque},
	hash::Hash,
	mem,
};

pub fn topological_sort<N, IN>(
	roots: &[N],
	mut successors: impl FnMut(&N) -> IN,
) -> Result<Vec<N>, N>
where
	N: Clone + Eq + Hash,
	IN: IntoIterator<Item = N>,
{
	let mut marked = HashSet::with_capacity(roots.len());
	let mut temp = HashSet::new();
	let mut sorted = VecDeque::with_capacity(roots.len());
	let mut roots = roots.iter().cloned().collect::<HashSet<_>>();
	while let Some(node) = roots.iter().next().cloned() {
		temp.clear();
		visit(
			&node,
			&mut successors,
			&mut roots,
			&mut marked,
			&mut temp,
			&mut sorted,
		)?;
	}

	Ok(sorted.into_iter().collect())
}

pub fn topological_sort_into_groups<N, IN>(
	nodes: &[N],
	mut successors: impl FnMut(&N) -> IN,
) -> Result<Vec<Vec<N>>, (Vec<Vec<N>>, Vec<N>)>
where
	N: Clone + Eq + Hash,
	IN: IntoIterator<Item = N>,
{
	if nodes.is_empty() {
		return Ok(Vec::new());
	}

	let mut succs_map = HashMap::with_capacity(nodes.len());
	let mut preds_map = HashMap::with_capacity(nodes.len());
	for node in nodes {
		succs_map.insert(
			node.clone(),
			successors(node).into_iter().collect::<HashSet<_>>(),
		);
		preds_map.insert(node.clone(), 0usize);
	}
	for succs in succs_map.values() {
		for succ in succs {
			*preds_map.get_mut(succ).unwrap() += 1;
		}
	}

	let mut groups = Vec::new();
	let mut prev_group = preds_map
		.iter()
		.filter(|&(_, num_preds)| matches!(*num_preds, 0))
		.map(|(node, _)| node.clone())
		.collect::<Vec<_>>();
	if prev_group.is_empty() {
		let remaining = preds_map.into_keys().collect();
		return Err((Vec::new(), remaining));
	}
	for node in &prev_group {
		preds_map.remove(node);
	}
	while !preds_map.is_empty() {
		let mut next_group = Vec::new();
		for node in &prev_group {
			for succ in &succs_map[node] {
				{
					let num_preds = preds_map.get_mut(succ).unwrap();
					*num_preds -= 1;
					if *num_preds > 0 {
						continue;
					}
				}

				next_group.push(preds_map.remove_entry(succ).unwrap().0);
			}
		}

		groups.push(mem::replace(&mut prev_group, next_group));
		if prev_group.is_empty() {
			let remaining = preds_map.into_keys().collect();
			return Err((groups, remaining));
		}
	}

	groups.push(prev_group);
	Ok(groups)
}

fn visit<N, IN>(
	node: &N,
	successors: &mut impl FnMut(&N) -> IN,
	unmarked: &mut HashSet<N>,
	marked: &mut HashSet<N>,
	temp: &mut HashSet<N>,
	sorted: &mut VecDeque<N>,
) -> Result<(), N>
where
	N: Clone + Eq + Hash,
	IN: IntoIterator<Item = N>,
{
	unmarked.remove(node);
	if marked.contains(node) {
		return Ok(());
	}

	if temp.contains(node) {
		return Err(node.clone());
	}

	temp.insert(node.clone());
	for n in successors(node) {
		visit(&n, successors, unmarked, marked, temp, sorted)?;
	}

	marked.insert(node.clone());
	sorted.push_front(node.clone());

	Ok(())
}
