///
//  Generated code. Do not modify.
//  source: angkorim.proto
//
// @dart = 2.12
// ignore_for_file: annotate_overrides,camel_case_types,unnecessary_const,non_constant_identifier_names,library_prefixes,unused_import,unused_shown_name,return_of_invalid_type,unnecessary_this,prefer_final_fields

import 'dart:async' as $async;

import 'dart:core' as $core;

import 'package:grpc/service_api.dart' as $grpc;
export 'angkorim.pb.dart';

class SessionServiceClient extends $grpc.Client {
  SessionServiceClient($grpc.ClientChannel channel,
      {$grpc.CallOptions? options,
      $core.Iterable<$grpc.ClientInterceptor>? interceptors})
      : super(channel, options: options, interceptors: interceptors);
}

abstract class SessionServiceBase extends $grpc.Service {
  $core.String get $name => 'protocol.SessionService';

  SessionServiceBase() {}
}

class MessageServiceClient extends $grpc.Client {
  MessageServiceClient($grpc.ClientChannel channel,
      {$grpc.CallOptions? options,
      $core.Iterable<$grpc.ClientInterceptor>? interceptors})
      : super(channel, options: options, interceptors: interceptors);
}

abstract class MessageServiceBase extends $grpc.Service {
  $core.String get $name => 'protocol.MessageService';

  MessageServiceBase() {}
}

class UserServiceClient extends $grpc.Client {
  UserServiceClient($grpc.ClientChannel channel,
      {$grpc.CallOptions? options,
      $core.Iterable<$grpc.ClientInterceptor>? interceptors})
      : super(channel, options: options, interceptors: interceptors);
}

abstract class UserServiceBase extends $grpc.Service {
  $core.String get $name => 'protocol.UserService';

  UserServiceBase() {}
}

class SyncServiceClient extends $grpc.Client {
  SyncServiceClient($grpc.ClientChannel channel,
      {$grpc.CallOptions? options,
      $core.Iterable<$grpc.ClientInterceptor>? interceptors})
      : super(channel, options: options, interceptors: interceptors);
}

abstract class SyncServiceBase extends $grpc.Service {
  $core.String get $name => 'protocol.SyncService';

  SyncServiceBase() {}
}
