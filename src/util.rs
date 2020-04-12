#[allow(dead_code)]
pub fn bracket<R, T, E, F, G, H>(acquire: F, release: G, consume: H) -> std::result::Result<T, E>
where
    F: FnOnce() -> std::result::Result<R, E>,
    G: FnOnce(R) -> (),
    H: FnOnce(&R) -> std::result::Result<T, E>,
{
    let resource = acquire()?;
    let result = consume(&resource);
    release(resource);
    result
}
