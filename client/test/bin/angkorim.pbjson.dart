///
//  Generated code. Do not modify.
//  source: angkorim.proto
//
// @dart = 2.12
// ignore_for_file: annotate_overrides,camel_case_types,unnecessary_const,non_constant_identifier_names,library_prefixes,unused_import,unused_shown_name,return_of_invalid_type,unnecessary_this,prefer_final_fields,deprecated_member_use_from_same_package

import 'dart:core' as $core;
import 'dart:convert' as $convert;
import 'dart:typed_data' as $typed_data;
@$core.Deprecated('Use responseCodeDescriptor instead')
const ResponseCode$json = const {
  '1': 'ResponseCode',
  '2': const [
    const {'1': 'UNKNOWN_ERROR', '2': 0},
    const {'1': 'REQUEST_ERROR', '2': 4096},
    const {'1': 'REQUEST_SUCCESS', '2': 8192},
  ],
};

/// Descriptor for `ResponseCode`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List responseCodeDescriptor = $convert.base64Decode('CgxSZXNwb25zZUNvZGUSEQoNVU5LTk9XTl9FUlJPUhAAEhIKDVJFUVVFU1RfRVJST1IQgCASFAoPUkVRVUVTVF9TVUNDRVNTEIBA');
@$core.Deprecated('Use commandDescriptor instead')
const Command$json = const {
  '1': 'Command',
  '2': const [
    const {'1': 'CMD_UNKNOWN', '2': 0},
    const {'1': 'CMD_SIGNIN', '2': 1},
    const {'1': 'CMD_SEND_MSG', '2': 2},
    const {'1': 'CMD_SUBSCRIBE_TOPIC', '2': 3},
  ],
};

/// Descriptor for `Command`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List commandDescriptor = $convert.base64Decode('CgdDb21tYW5kEg8KC0NNRF9VTktOT1dOEAASDgoKQ01EX1NJR05JThABEhAKDENNRF9TRU5EX01TRxACEhcKE0NNRF9TVUJTQ1JJQkVfVE9QSUMQAw==');
@$core.Deprecated('Use deviceTypeDescriptor instead')
const DeviceType$json = const {
  '1': 'DeviceType',
  '2': const [
    const {'1': 'WEB', '2': 0},
    const {'1': 'ANDRIOD', '2': 1},
    const {'1': 'IOS', '2': 2},
    const {'1': 'CLI', '2': 3},
    const {'1': 'UNKNOWN', '2': 4},
  ],
};

/// Descriptor for `DeviceType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List deviceTypeDescriptor = $convert.base64Decode('CgpEZXZpY2VUeXBlEgcKA1dFQhAAEgsKB0FORFJJT0QQARIHCgNJT1MQAhIHCgNDTEkQAxILCgdVTktOT1dOEAQ=');
@$core.Deprecated('Use messageTypeDescriptor instead')
const MessageType$json = const {
  '1': 'MessageType',
  '2': const [
    const {'1': 'MSG_UNKNOWN', '2': 0},
    const {'1': 'MSG_TEXT', '2': 4096},
  ],
};

/// Descriptor for `MessageType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List messageTypeDescriptor = $convert.base64Decode('CgtNZXNzYWdlVHlwZRIPCgtNU0dfVU5LTk9XThAAEg0KCE1TR19URVhUEIAg');
@$core.Deprecated('Use messageStatusDescriptor instead')
const MessageStatus$json = const {
  '1': 'MessageStatus',
  '2': const [
    const {'1': 'DELIVERED', '2': 0},
    const {'1': 'SEEN', '2': 256},
  ],
};

/// Descriptor for `MessageStatus`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List messageStatusDescriptor = $convert.base64Decode('Cg1NZXNzYWdlU3RhdHVzEg0KCURFTElWRVJFRBAAEgkKBFNFRU4QgAI=');
@$core.Deprecated('Use receiverTypeDescriptor instead')
const ReceiverType$json = const {
  '1': 'ReceiverType',
  '2': const [
    const {'1': 'RECEV_UNKOWN', '2': 0},
    const {'1': 'RECEV_P2P', '2': 1},
    const {'1': 'RECEV_GROUP', '2': 2},
    const {'1': 'RECEV_CHANNEL', '2': 3},
    const {'1': 'RECEV_SMALL_GROUP', '2': 4},
  ],
};

/// Descriptor for `ReceiverType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List receiverTypeDescriptor = $convert.base64Decode('CgxSZWNlaXZlclR5cGUSEAoMUkVDRVZfVU5LT1dOEAASDQoJUkVDRVZfUDJQEAESDwoLUkVDRVZfR1JPVVAQAhIRCg1SRUNFVl9DSEFOTkVMEAMSFQoRUkVDRVZfU01BTExfR1JPVVAQBA==');
@$core.Deprecated('Use emptyMsgDescriptor instead')
const EmptyMsg$json = const {
  '1': 'EmptyMsg',
};

/// Descriptor for `EmptyMsg`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List emptyMsgDescriptor = $convert.base64Decode('CghFbXB0eU1zZw==');
@$core.Deprecated('Use requestDescriptor instead')
const Request$json = const {
  '1': 'Request',
  '2': const [
    const {'1': 'cmd', '3': 1, '4': 1, '5': 14, '6': '.protocol.Command', '10': 'cmd'},
    const {'1': 'data', '3': 2, '4': 1, '5': 12, '10': 'data'},
  ],
};

/// Descriptor for `Request`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List requestDescriptor = $convert.base64Decode('CgdSZXF1ZXN0EiMKA2NtZBgBIAEoDjIRLnByb3RvY29sLkNvbW1hbmRSA2NtZBISCgRkYXRhGAIgASgMUgRkYXRh');
@$core.Deprecated('Use responseDescriptor instead')
const Response$json = const {
  '1': 'Response',
  '2': const [
    const {'1': 'cmd', '3': 1, '4': 1, '5': 14, '6': '.protocol.Command', '10': 'cmd'},
    const {'1': 'code', '3': 2, '4': 1, '5': 14, '6': '.protocol.ResponseCode', '10': 'code'},
    const {'1': 'message', '3': 3, '4': 1, '5': 9, '10': 'message'},
    const {'1': 'data', '3': 4, '4': 1, '5': 12, '10': 'data'},
  ],
};

/// Descriptor for `Response`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List responseDescriptor = $convert.base64Decode('CghSZXNwb25zZRIjCgNjbWQYASABKA4yES5wcm90b2NvbC5Db21tYW5kUgNjbWQSKgoEY29kZRgCIAEoDjIWLnByb3RvY29sLlJlc3BvbnNlQ29kZVIEY29kZRIYCgdtZXNzYWdlGAMgASgJUgdtZXNzYWdlEhIKBGRhdGEYBCABKAxSBGRhdGE=');
@$core.Deprecated('Use signInRequestDescriptor instead')
const SignInRequest$json = const {
  '1': 'SignInRequest',
  '2': const [
    const {'1': 'phone_number', '3': 1, '4': 1, '5': 9, '10': 'phoneNumber'},
    const {'1': 'code', '3': 2, '4': 1, '5': 9, '10': 'code'},
    const {'1': 'device_id', '3': 3, '4': 1, '5': 3, '10': 'deviceId'},
  ],
};

/// Descriptor for `SignInRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List signInRequestDescriptor = $convert.base64Decode('Cg1TaWduSW5SZXF1ZXN0EiEKDHBob25lX251bWJlchgBIAEoCVILcGhvbmVOdW1iZXISEgoEY29kZRgCIAEoCVIEY29kZRIbCglkZXZpY2VfaWQYAyABKANSCGRldmljZUlk');
@$core.Deprecated('Use signInResponseDescriptor instead')
const SignInResponse$json = const {
  '1': 'SignInResponse',
  '2': const [
    const {'1': 'token', '3': 1, '4': 1, '5': 9, '10': 'token'},
  ],
};

/// Descriptor for `SignInResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List signInResponseDescriptor = $convert.base64Decode('Cg5TaWduSW5SZXNwb25zZRIUCgV0b2tlbhgBIAEoCVIFdG9rZW4=');
@$core.Deprecated('Use subscribeRequestDescriptor instead')
const SubscribeRequest$json = const {
  '1': 'SubscribeRequest',
  '2': const [
    const {'1': 'topic', '3': 1, '4': 1, '5': 9, '10': 'topic'},
  ],
};

/// Descriptor for `SubscribeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List subscribeRequestDescriptor = $convert.base64Decode('ChBTdWJzY3JpYmVSZXF1ZXN0EhQKBXRvcGljGAEgASgJUgV0b3BpYw==');
@$core.Deprecated('Use subscribeResponseDescriptor instead')
const SubscribeResponse$json = const {
  '1': 'SubscribeResponse',
  '2': const [
    const {'1': 'topic', '3': 1, '4': 1, '5': 9, '10': 'topic'},
  ],
};

/// Descriptor for `SubscribeResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List subscribeResponseDescriptor = $convert.base64Decode('ChFTdWJzY3JpYmVSZXNwb25zZRIUCgV0b3BpYxgBIAEoCVIFdG9waWM=');
@$core.Deprecated('Use registerDeviceRequestDescriptor instead')
const RegisterDeviceRequest$json = const {
  '1': 'RegisterDeviceRequest',
  '2': const [
    const {'1': 'device_id', '3': 1, '4': 1, '5': 9, '10': 'deviceId'},
    const {'1': 'device_type', '3': 2, '4': 1, '5': 14, '6': '.protocol.DeviceType', '10': 'deviceType'},
    const {'1': 'user_agent', '3': 3, '4': 1, '5': 9, '10': 'userAgent'},
  ],
};

/// Descriptor for `RegisterDeviceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List registerDeviceRequestDescriptor = $convert.base64Decode('ChVSZWdpc3RlckRldmljZVJlcXVlc3QSGwoJZGV2aWNlX2lkGAEgASgJUghkZXZpY2VJZBI1CgtkZXZpY2VfdHlwZRgCIAEoDjIULnByb3RvY29sLkRldmljZVR5cGVSCmRldmljZVR5cGUSHQoKdXNlcl9hZ2VudBgDIAEoCVIJdXNlckFnZW50');
@$core.Deprecated('Use registerDeviceResponseDescriptor instead')
const RegisterDeviceResponse$json = const {
  '1': 'RegisterDeviceResponse',
  '2': const [
    const {'1': 'device_id', '3': 1, '4': 1, '5': 9, '10': 'deviceId'},
    const {'1': 'user_agent', '3': 2, '4': 1, '5': 9, '10': 'userAgent'},
  ],
};

/// Descriptor for `RegisterDeviceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List registerDeviceResponseDescriptor = $convert.base64Decode('ChZSZWdpc3RlckRldmljZVJlc3BvbnNlEhsKCWRldmljZV9pZBgBIAEoCVIIZGV2aWNlSWQSHQoKdXNlcl9hZ2VudBgCIAEoCVIJdXNlckFnZW50');
@$core.Deprecated('Use userDescriptor instead')
const User$json = const {
  '1': 'User',
  '2': const [
    const {'1': 'user_id', '3': 1, '4': 1, '5': 9, '10': 'userId'},
    const {'1': 'nickname', '3': 2, '4': 1, '5': 9, '10': 'nickname'},
    const {'1': 'gender', '3': 3, '4': 1, '5': 5, '10': 'gender'},
    const {'1': 'avatar_url', '3': 4, '4': 1, '5': 9, '10': 'avatarUrl'},
    const {'1': 'extra', '3': 5, '4': 1, '5': 9, '10': 'extra'},
    const {'1': 'create_time', '3': 6, '4': 1, '5': 3, '10': 'createTime'},
    const {'1': 'update_time', '3': 7, '4': 1, '5': 3, '10': 'updateTime'},
  ],
};

/// Descriptor for `User`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List userDescriptor = $convert.base64Decode('CgRVc2VyEhcKB3VzZXJfaWQYASABKAlSBnVzZXJJZBIaCghuaWNrbmFtZRgCIAEoCVIIbmlja25hbWUSFgoGZ2VuZGVyGAMgASgFUgZnZW5kZXISHQoKYXZhdGFyX3VybBgEIAEoCVIJYXZhdGFyVXJsEhQKBWV4dHJhGAUgASgJUgVleHRyYRIfCgtjcmVhdGVfdGltZRgGIAEoA1IKY3JlYXRlVGltZRIfCgt1cGRhdGVfdGltZRgHIAEoA1IKdXBkYXRlVGltZQ==');
@$core.Deprecated('Use messageDescriptor instead')
const Message$json = const {
  '1': 'Message',
  '2': const [
    const {'1': 'sender', '3': 1, '4': 1, '5': 11, '6': '.protocol.User', '10': 'sender'},
    const {'1': 'receiver_type', '3': 2, '4': 1, '5': 14, '6': '.protocol.ReceiverType', '10': 'receiverType'},
    const {'1': 'to_id', '3': 3, '4': 1, '5': 9, '10': 'toId'},
    const {'1': 'to_user_ids', '3': 4, '4': 3, '5': 3, '10': 'toUserIds'},
    const {'1': 'message_type', '3': 5, '4': 1, '5': 14, '6': '.protocol.MessageType', '10': 'messageType'},
    const {'1': 'message_content', '3': 6, '4': 1, '5': 12, '10': 'messageContent'},
    const {'1': 'seq', '3': 7, '4': 1, '5': 3, '10': 'seq'},
    const {'1': 'create_time', '3': 8, '4': 1, '5': 3, '10': 'createTime'},
    const {'1': 'status', '3': 9, '4': 1, '5': 14, '6': '.protocol.MessageStatus', '10': 'status'},
  ],
};

/// Descriptor for `Message`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List messageDescriptor = $convert.base64Decode('CgdNZXNzYWdlEiYKBnNlbmRlchgBIAEoCzIOLnByb3RvY29sLlVzZXJSBnNlbmRlchI7Cg1yZWNlaXZlcl90eXBlGAIgASgOMhYucHJvdG9jb2wuUmVjZWl2ZXJUeXBlUgxyZWNlaXZlclR5cGUSEwoFdG9faWQYAyABKAlSBHRvSWQSHgoLdG9fdXNlcl9pZHMYBCADKANSCXRvVXNlcklkcxI4CgxtZXNzYWdlX3R5cGUYBSABKA4yFS5wcm90b2NvbC5NZXNzYWdlVHlwZVILbWVzc2FnZVR5cGUSJwoPbWVzc2FnZV9jb250ZW50GAYgASgMUg5tZXNzYWdlQ29udGVudBIQCgNzZXEYByABKANSA3NlcRIfCgtjcmVhdGVfdGltZRgIIAEoA1IKY3JlYXRlVGltZRIvCgZzdGF0dXMYCSABKA4yFy5wcm90b2NvbC5NZXNzYWdlU3RhdHVzUgZzdGF0dXM=');
@$core.Deprecated('Use messageRequestDescriptor instead')
const MessageRequest$json = const {
  '1': 'MessageRequest',
  '2': const [
    const {'1': 'message', '3': 1, '4': 1, '5': 11, '6': '.protocol.Message', '10': 'message'},
  ],
};

/// Descriptor for `MessageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List messageRequestDescriptor = $convert.base64Decode('Cg5NZXNzYWdlUmVxdWVzdBIrCgdtZXNzYWdlGAEgASgLMhEucHJvdG9jb2wuTWVzc2FnZVIHbWVzc2FnZQ==');
@$core.Deprecated('Use simpleMessageRequestDescriptor instead')
const SimpleMessageRequest$json = const {
  '1': 'SimpleMessageRequest',
  '2': const [
    const {'1': 'message', '3': 1, '4': 1, '5': 9, '10': 'message'},
    const {'1': 'topic', '3': 2, '4': 1, '5': 9, '10': 'topic'},
  ],
};

/// Descriptor for `SimpleMessageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List simpleMessageRequestDescriptor = $convert.base64Decode('ChRTaW1wbGVNZXNzYWdlUmVxdWVzdBIYCgdtZXNzYWdlGAEgASgJUgdtZXNzYWdlEhQKBXRvcGljGAIgASgJUgV0b3BpYw==');
@$core.Deprecated('Use simpleMessageResponseDescriptor instead')
const SimpleMessageResponse$json = const {
  '1': 'SimpleMessageResponse',
  '2': const [
    const {'1': 'message', '3': 1, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `SimpleMessageResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List simpleMessageResponseDescriptor = $convert.base64Decode('ChVTaW1wbGVNZXNzYWdlUmVzcG9uc2USGAoHbWVzc2FnZRgBIAEoCVIHbWVzc2FnZQ==');
