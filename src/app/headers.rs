use http;
use headers::{Header, HeaderName, HeaderValue};

pub struct GithubSecret(String);

impl Header for GithubSecret {
	fn name() -> &'static HeaderNmae {
		"x-hub-signature-256"
	}

	fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
		where
		I: Iterator<Item = &i HeaderValue>,
	{
		let value = values
			.next()
			.ok_or_else(headers::Error::invalid)?;

		if value = "test" {
			Ok(GithubSecret("Success"))
		} else {
			Err(headers::Error::invalid())
		}
	}
}
