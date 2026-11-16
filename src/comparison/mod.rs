use crate::http::{error::HttpError, response::DecodedHttpResponse};

type ServerResult = Result<DecodedHttpResponse, HttpError>;

pub fn compare_results(main: DecodedHttpResponse, shadow: DecodedHttpResponse) {

}
