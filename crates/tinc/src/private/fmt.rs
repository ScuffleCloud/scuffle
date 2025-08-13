pub struct FuncFmt<F: Fn(&mut std::fmt::Formatter) -> std::fmt::Result>(pub F);

impl<F> std::fmt::Display for FuncFmt<F>
where
    F: Fn(&mut std::fmt::Formatter) -> std::fmt::Result,
{
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0)(f)
    }
}
impl<F> std::fmt::Debug for FuncFmt<F>
where
    F: Fn(&mut std::fmt::Formatter) -> std::fmt::Result,
{
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0)(f)
    }
}
