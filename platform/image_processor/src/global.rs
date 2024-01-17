use common::s3::Bucket;

use crate::config::ImageProcessorConfig;

pub trait ImageProcessorState {
	fn s3_source_bucket(&self) -> &Bucket;
	fn s3_target_bucket(&self) -> &Bucket;
}

pub trait ImageProcessorGlobal:
	common::global::GlobalCtx
	+ common::global::GlobalConfigProvider<ImageProcessorConfig>
	+ common::global::GlobalNats
	+ common::global::GlobalDb
	+ common::global::GlobalConfig
	+ ImageProcessorState
	+ Send
	+ Sync
	+ 'static
{
}

impl<T> ImageProcessorGlobal for T where
	T: common::global::GlobalCtx
		+ common::global::GlobalConfigProvider<ImageProcessorConfig>
		+ common::global::GlobalNats
		+ common::global::GlobalDb
		+ common::global::GlobalConfig
		+ ImageProcessorState
		+ Send
		+ Sync
		+ 'static
{
}
