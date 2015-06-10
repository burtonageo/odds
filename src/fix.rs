
/// Fixpoint combinator for rust closures, generalized over the return type.
///
/// The **Fix** only supports direct function call notation with the nightly channel.
///
/// ```
/// use odds::Fix;
///
/// let c = |f: Fix<i32, _>, x| if x == 0 { 1 } else { x * f.call(x - 1) };
/// let fact = Fix(&c);
/// assert_eq!(fact.call(5), 120);
///
/// let data = &[true, false];
/// let all_true = |f: Fix<_, _>, x| {
///     let x: &[_] = x;
///     x.len() == 0 || x[0] && f.call(&x[1..])
/// };
/// let all = Fix(&all_true);
/// assert_eq!(all.call(data), false);
/// ```
pub struct Fix<'a, T, R>(pub &'a Fn(Fix<T, R>, T) -> R);

impl<'a, T, R> Fix<'a, T, R> {
    pub fn call(&self, arg: T) -> R {
        let f = *self;
        f.0(f, arg)
    }
}

impl<'a, T, R> Clone for Fix<'a, T, R> {
    fn clone(&self) -> Self { *self }
}

impl<'a, T, R> Copy for Fix<'a, T, R> { }
