use regex::Regex;
use vmm_testing::{arbitrary, run_test};

fn arbitrary_regex(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<String> {
	let choices: &[fn(&mut arbitrary::Unstructured<'_>) -> arbitrary::Result<String>] = &[
		|_u| Ok("a".to_owned()),
		|_u| Ok("b".to_owned()),
		|u| arbitrary_regex(u).map(|r| format!("({r})*")),
		|u| {
			let l = arbitrary_regex(u)?;
			let r = arbitrary_regex(u)?;
			Ok(format!("{l}{r}"))
		},
		|u| {
			let l = arbitrary_regex(u)?;
			let r = arbitrary_regex(u)?;
			Ok(format!("({l})|({r})"))
		},
	];

	u.choose(choices)?(u)
}

fn main() {
	_ = run_test(|u| {
		let r = arbitrary_regex(u)?;
		if let Ok(regex) = Regex::new(&format!("^({r})$"))
			&& regex.is_match("abba")
			&& !regex.is_match("baab")
		{
			eprintln!("{r}");
			panic!()
		}

		Ok(())
	})
	.budget_ms(10_000)
	.seed(0x7abc_b628_0000_0020)
	.minimize();
}
