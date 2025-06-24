use core::convert::TryInto;

pub trait Convert
where
	Self: Sized,
{
	fn convert<T>(self) -> T
	where
		Self: Into<T>,
		T: Sized,
	{
		Into::into(self)
	}
}

impl<T> Convert for T {}

pub trait TryConvert
where
	Self: Sized,
{
	fn try_convert<T>(self) -> Result<T, Self::Error>
	where
		Self: TryInto<T>,
		T: Sized,
	{
		TryInto::try_into(self)
	}
}

impl<T> TryConvert for T {}
