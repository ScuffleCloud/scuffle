pub trait AwsSesInterface {
    fn aws_ses_client(&self) -> &aws_sdk_sesv2::Client;
}
