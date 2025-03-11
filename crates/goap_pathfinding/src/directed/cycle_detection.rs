pub fn floyd<T>(start: T, successor: impl Fn(T) -> T) -> (usize, T, usize)
where
	T: Clone + PartialEq,
{
	let mut tortoise = successor(start.clone());
	let mut hare = successor(successor(start.clone()));
	while tortoise != hare {
		(tortoise, hare) = (successor(tortoise), successor(successor(hare)));
	}
	let mut mu = 0;
	tortoise = start;
	while tortoise != hare {
		(tortoise, hare, mu) = (successor(tortoise), successor(hare), mu + 1);
	}
	let mut lam = 1;
	hare = successor(tortoise.clone());
	while tortoise != hare {
		(hare, lam) = (successor(hare), lam + 1);
	}

	(lam, tortoise, mu)
}

pub fn brent<T>(start: T, successor: impl Fn(T) -> T) -> (usize, T, usize)
where
	T: Clone + PartialEq,
{
	let mut power = 1;
	let mut lam = 1;
	let mut tortoise = start.clone();
	let mut hare = successor(start.clone());
	while tortoise != hare {
		if power == lam {
			(tortoise, power, lam) = (hare.clone(), power * 2, 0);
		}
		(hare, lam) = (successor(hare), lam + 1);
	}
	let mut mu = 0;
	(tortoise, hare) = (start.clone(), (0..lam).fold(start, |x, _| successor(x)));
	while tortoise != hare {
		(tortoise, hare, mu) = (successor(tortoise), successor(hare), mu + 1);
	}

	(lam, hare, mu)
}

#[cfg(test)]
mod tests {
	use super::{brent, floyd};

	#[test]
	fn floyd_works() {
		assert_eq!(floyd(-10, |x| (x + 5) % 6 + 3), (3, 6, 2));
	}

	#[test]
	fn brent_works() {
		assert_eq!(brent(-10, |x| (x + 5) % 6 + 3), (3, 6, 2));
	}
}
