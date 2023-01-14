///
//  Generated code. Do not modify.
//  source: angkorim.proto
//
// @dart = 2.12
// ignore_for_file: annotate_overrides,camel_case_types,unnecessary_const,non_constant_identifier_names,library_prefixes,unused_import,unused_shown_name,return_of_invalid_type,unnecessary_this,prefer_final_fields

// ignore_for_file: UNDEFINED_SHOWN_NAME
import 'dart:core' as $core;
import 'package:protobuf/protobuf.dart' as $pb;

class ResponseCode extends $pb.ProtobufEnum {
  static const ResponseCode UNKNOWN_ERROR = ResponseCode._(0, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'UNKNOWN_ERROR');
  static const ResponseCode REQUEST_ERROR = ResponseCode._(4096, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'REQUEST_ERROR');
  static const ResponseCode REQUEST_SUCCESS = ResponseCode._(8192, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'REQUEST_SUCCESS');

  static const $core.List<ResponseCode> values = <ResponseCode> [
    UNKNOWN_ERROR,
    REQUEST_ERROR,
    REQUEST_SUCCESS,
  ];

  static final $core.Map<$core.int, ResponseCode> _byValue = $pb.ProtobufEnum.initByValue(values);
  static ResponseCode? valueOf($core.int value) => _byValue[value];

  const ResponseCode._($core.int v, $core.String n) : super(v, n);
}

class Command extends $pb.ProtobufEnum {
  static const Command CMD_UNKNOWN = Command._(0, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'CMD_UNKNOWN');
  static const Command CMD_SIGNIN = Command._(1, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'CMD_SIGNIN');
  static const Command CMD_SEND_MSG = Command._(2, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'CMD_SEND_MSG');
  static const Command CMD_SUBSCRIBE_TOPIC = Command._(3, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'CMD_SUBSCRIBE_TOPIC');
  static const Command CMD_CLUSTER_UPDATE_MEMBERS = Command._(4, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'CMD_CLUSTER_UPDATE_MEMBERS');

  static const $core.List<Command> values = <Command> [
    CMD_UNKNOWN,
    CMD_SIGNIN,
    CMD_SEND_MSG,
    CMD_SUBSCRIBE_TOPIC,
    CMD_CLUSTER_UPDATE_MEMBERS,
  ];

  static final $core.Map<$core.int, Command> _byValue = $pb.ProtobufEnum.initByValue(values);
  static Command? valueOf($core.int value) => _byValue[value];

  const Command._($core.int v, $core.String n) : super(v, n);
}

class DeviceType extends $pb.ProtobufEnum {
  static const DeviceType WEB = DeviceType._(0, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'WEB');
  static const DeviceType ANDRIOD = DeviceType._(1, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'ANDRIOD');
  static const DeviceType IOS = DeviceType._(2, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'IOS');
  static const DeviceType CLI = DeviceType._(3, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'CLI');
  static const DeviceType UNKNOWN = DeviceType._(4, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'UNKNOWN');

  static const $core.List<DeviceType> values = <DeviceType> [
    WEB,
    ANDRIOD,
    IOS,
    CLI,
    UNKNOWN,
  ];

  static final $core.Map<$core.int, DeviceType> _byValue = $pb.ProtobufEnum.initByValue(values);
  static DeviceType? valueOf($core.int value) => _byValue[value];

  const DeviceType._($core.int v, $core.String n) : super(v, n);
}

class MessageType extends $pb.ProtobufEnum {
  static const MessageType MSG_UNKNOWN = MessageType._(0, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'MSG_UNKNOWN');
  static const MessageType MSG_TEXT = MessageType._(4096, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'MSG_TEXT');

  static const $core.List<MessageType> values = <MessageType> [
    MSG_UNKNOWN,
    MSG_TEXT,
  ];

  static final $core.Map<$core.int, MessageType> _byValue = $pb.ProtobufEnum.initByValue(values);
  static MessageType? valueOf($core.int value) => _byValue[value];

  const MessageType._($core.int v, $core.String n) : super(v, n);
}

class MessageStatus extends $pb.ProtobufEnum {
  static const MessageStatus DELIVERED = MessageStatus._(0, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'DELIVERED');
  static const MessageStatus SEEN = MessageStatus._(256, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'SEEN');

  static const $core.List<MessageStatus> values = <MessageStatus> [
    DELIVERED,
    SEEN,
  ];

  static final $core.Map<$core.int, MessageStatus> _byValue = $pb.ProtobufEnum.initByValue(values);
  static MessageStatus? valueOf($core.int value) => _byValue[value];

  const MessageStatus._($core.int v, $core.String n) : super(v, n);
}

class ReceiverType extends $pb.ProtobufEnum {
  static const ReceiverType RECEV_UNKOWN = ReceiverType._(0, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'RECEV_UNKOWN');
  static const ReceiverType RECEV_P2P = ReceiverType._(1, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'RECEV_P2P');
  static const ReceiverType RECEV_GROUP = ReceiverType._(2, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'RECEV_GROUP');
  static const ReceiverType RECEV_CHANNEL = ReceiverType._(3, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'RECEV_CHANNEL');
  static const ReceiverType RECEV_SMALL_GROUP = ReceiverType._(4, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'RECEV_SMALL_GROUP');

  static const $core.List<ReceiverType> values = <ReceiverType> [
    RECEV_UNKOWN,
    RECEV_P2P,
    RECEV_GROUP,
    RECEV_CHANNEL,
    RECEV_SMALL_GROUP,
  ];

  static final $core.Map<$core.int, ReceiverType> _byValue = $pb.ProtobufEnum.initByValue(values);
  static ReceiverType? valueOf($core.int value) => _byValue[value];

  const ReceiverType._($core.int v, $core.String n) : super(v, n);
}

class NodeStatus extends $pb.ProtobufEnum {
  static const NodeStatus UNKNOWN_NODE = NodeStatus._(0, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'UNKNOWN_NODE');
  static const NodeStatus ALIVE_NODE = NodeStatus._(4096, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'ALIVE_NODE');
  static const NodeStatus SUSPECTED_NODE = NodeStatus._(8192, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'SUSPECTED_NODE');
  static const NodeStatus SUSPECTED_DEAD = NodeStatus._(12288, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'SUSPECTED_DEAD');

  static const $core.List<NodeStatus> values = <NodeStatus> [
    UNKNOWN_NODE,
    ALIVE_NODE,
    SUSPECTED_NODE,
    SUSPECTED_DEAD,
  ];

  static final $core.Map<$core.int, NodeStatus> _byValue = $pb.ProtobufEnum.initByValue(values);
  static NodeStatus? valueOf($core.int value) => _byValue[value];

  const NodeStatus._($core.int v, $core.String n) : super(v, n);
}

