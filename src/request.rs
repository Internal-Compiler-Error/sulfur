pub(crate) mod http;

#[non_exhaustive]
pub enum Request<T> {
    HTTP(T),
}
