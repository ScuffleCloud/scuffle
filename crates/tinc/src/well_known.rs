//! Well Known Protobuf types

/// Wellknown types for Prost
#[cfg(feature = "prost")]
pub mod prost {
    /// Protobuf `google.protobuf.Timestamp`
    pub type Timestamp = prost_types::Timestamp;
    /// Protobuf `google.protobuf.Duration`
    pub type Duration = prost_types::Duration;
    /// Protobuf `google.protobuf.Struct`
    pub type Struct = prost_types::Struct;
    /// Protobuf `google.protobuf.Value`
    pub type Value = prost_types::Value;
    /// Protobuf `google.protobuf.Empty`
    pub type Empty = ();
    /// Protobuf `google.protobuf.ListValue`
    pub type ListValue = prost_types::ListValue;
    /// Protobuf `google.protobuf.Any`
    pub type Any = prost_types::Any;
    /// Protobuf `google.protobuf.BoolValue`
    pub type BoolValue = bool;
    /// Protobuf `google.protobuf.Int32Value`
    pub type Int32Value = i32;
    /// Protobuf `google.protobuf.Int64Value`
    pub type Int64Value = i64;
    /// Protobuf `google.protobuf.UInt32Value`
    pub type UInt32Value = u32;
    /// Protobuf `google.protobuf.UInt64Value`
    pub type UInt64Value = u64;
    /// Protobuf `google.protobuf.FloatValue`
    pub type FloatValue = f32;
    /// Protobuf `google.protobuf.DoubleValue`
    pub type DoubleValue = f64;
    /// Protobuf `google.protobuf.StringValue`
    pub type String = std::string::String;
    /// Protobuf `google.protobuf.BytesValue`
    pub type Bytes = bytes::Bytes;
}
