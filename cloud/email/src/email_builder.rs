use ext_traits::{OptionExt, ResultExt};
use mail_builder::headers::address::Address;

pub(crate) fn build_raw_email(email: pb::scufflecloud::email::v1::Email) -> Result<Vec<u8>, tonic::Status> {
    let from = email.from.require("from")?;
    let to = email.to.require("to")?;

    mail_builder::MessageBuilder::new()
        .from(Address::new_address(from.name, from.address))
        .to(Address::new_address(to.name, to.address))
        .subject(email.subject)
        .text_body(email.text)
        .html_body(email.html)
        .write_to_vec()
        .into_tonic_internal_err("failed to build raw email")
}
