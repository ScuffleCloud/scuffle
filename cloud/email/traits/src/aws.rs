pub trait AwsInterface {
    fn aws_region(&self) -> &str;
    fn aws_ses_req_signer(&self) -> &reqsign::Signer<reqsign::aws::Credential>;
}
