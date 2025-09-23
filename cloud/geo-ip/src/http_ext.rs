use ext_traits::OptionExt;

use crate::middleware::IpAddressInfo;

pub trait GeoIpRequestExt: ext_traits::RequestExt {
    fn ip_address_info(&self) -> Result<IpAddressInfo, tonic::Status> {
        self.extensions()
            .get::<IpAddressInfo>()
            .copied()
            .into_tonic_internal_err("missing IpAddressInfo extension")
    }
}

impl<T: ext_traits::RequestExt> GeoIpRequestExt for T {}
