use failure::Fail;

#[derive(Debug, Fail)]
pub enum RenderError {
    #[fail(display = "encode error: {}", _0)]
    Encode(#[cause] png::EncodingError),
}
