use std::{collections::HashSet, hash::Hash};

pub fn maximal_cliques_collect<N, IN>(
	vertices: IN,
	connected: &mut impl FnMut(&N, &N) -> bool,
) -> Vec<HashSet<N>>
where
	N: Clone + Eq + Hash,
	IN: IntoIterator<Item = N>,
{
	let mut result = Vec::new();
	let mut consumer = |n: &HashSet<N>| result.push(n.to_owned());
	let mut remaining_nodes = vertices.into_iter().collect::<HashSet<_>>();
	bron_kerbosch(
		connected,
		&HashSet::new(),
		&mut remaining_nodes,
		&mut HashSet::new(),
		&mut consumer,
	);
	result
}

pub fn maximal_cliques<N, IN>(
	vertices: IN,
	connected: &mut impl FnMut(&N, &N) -> bool,
	consumer: &mut impl FnMut(&HashSet<N>),
) where
	N: Clone + Eq + Hash,
	IN: IntoIterator<Item = N>,
{
	let mut remaining_nodes = vertices.into_iter().collect::<HashSet<_>>();
	bron_kerbosch(
		connected,
		&HashSet::new(),
		&mut remaining_nodes,
		&mut HashSet::new(),
		consumer,
	);
}

fn bron_kerbosch<N>(
	connected: &mut impl FnMut(&N, &N) -> bool,
	potential_clique: &HashSet<N>,
	remaining_nodes: &mut HashSet<N>,
	skip_nodes: &mut HashSet<N>,
	consumer: &mut impl FnMut(&HashSet<N>),
) where
	N: Clone + Hash + Eq,
{
	if remaining_nodes.is_empty() && skip_nodes.is_empty() {
		consumer(potential_clique);
		return;
	}

	let nodes_to_check = remaining_nodes.clone();
	for node in &nodes_to_check {
		let mut new_potential_clique = potential_clique.clone();
		new_potential_clique.insert(node.to_owned());

		let mut new_remaining_nodes = remaining_nodes
			.iter()
			.filter(|n| *n != node && connected(node, n))
			.cloned()
			.collect::<HashSet<_>>();

		let mut new_skip_list = skip_nodes
			.iter()
			.filter(|n| *n != node && connected(node, n))
			.cloned()
			.collect::<HashSet<_>>();

		bron_kerbosch(
			connected,
			&new_potential_clique,
			&mut new_remaining_nodes,
			&mut new_skip_list,
			consumer,
		);
		remaining_nodes.remove(node);
		skip_nodes.insert(node.to_owned());
	}
}
