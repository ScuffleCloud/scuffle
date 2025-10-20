mod device;
mod organization;
mod organization_invitation;
mod user;
mod user_email;
mod user_login;
mod user_mfa;

pub(crate) struct CoreSvc<G>(std::marker::PhantomData<G>);

impl<G> std::fmt::Debug for CoreSvc<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CoreSvc")
    }
}

impl<G> Clone for CoreSvc<G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<G> Copy for CoreSvc<G> {}

impl<G> Default for CoreSvc<G> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<G> CoreSvc<G> {
    pub(crate) fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}
