pub trait WebAuthnInterface {
    fn webauthn(&self) -> &webauthn_rs::Webauthn;
}
