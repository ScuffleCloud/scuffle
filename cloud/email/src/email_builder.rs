use ext_traits::{OptionExt, ResultExt};
use mail_builder::headers::address::Address;

pub(crate) fn build_raw_email(
    send_request: pb::scufflecloud::email::v1::SendEmailRequest,
) -> Result<Vec<u8>, tonic::Status> {
    let from = send_request.from.require("from")?;
    let to = send_request.to.require("to")?;

    mail_builder::MessageBuilder::new()
        .from(Address::new_address(from.name, from.address))
        .to(Address::new_address(to.name, to.address))
        .subject(send_request.subject)
        .text_body(send_request.text)
        .html_body(send_request.html)
        .write_to_vec()
        .into_tonic_internal_err("failed to build raw email")
}
