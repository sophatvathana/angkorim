///
//  Generated code. Do not modify.
//  source: angkorim.proto
//
// @dart = 2.12
// ignore_for_file: annotate_overrides,camel_case_types,unnecessary_const,non_constant_identifier_names,library_prefixes,unused_import,unused_shown_name,return_of_invalid_type,unnecessary_this,prefer_final_fields

import 'dart:async' as $async;

import 'dart:core' as $core;

import 'package:grpc/service_api.dart' as $grpc;
import 'angkorim.pb.dart' as $0;
export 'angkorim.pb.dart';

class ClusterClient extends $grpc.Client {
  static final _$seyHello =
      $grpc.ClientMethod<$0.RequestHello, $0.ResponseHello>(
          '/protocol.Cluster/SeyHello',
          ($0.RequestHello value) => value.writeToBuffer(),
          ($core.List<$core.int> value) => $0.ResponseHello.fromBuffer(value));

  ClusterClient($grpc.ClientChannel channel,
      {$grpc.CallOptions? options,
      $core.Iterable<$grpc.ClientInterceptor>? interceptors})
      : super(channel, options: options, interceptors: interceptors);

  $grpc.ResponseFuture<$0.ResponseHello> seyHello($0.RequestHello request,
      {$grpc.CallOptions? options}) {
    return $createUnaryCall(_$seyHello, request, options: options);
  }
}

abstract class ClusterServiceBase extends $grpc.Service {
  $core.String get $name => 'protocol.Cluster';

  ClusterServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.RequestHello, $0.ResponseHello>(
        'SeyHello',
        seyHello_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.RequestHello.fromBuffer(value),
        ($0.ResponseHello value) => value.writeToBuffer()));
  }

  $async.Future<$0.ResponseHello> seyHello_Pre(
      $grpc.ServiceCall call, $async.Future<$0.RequestHello> request) async {
    return seyHello(call, await request);
  }

  $async.Future<$0.ResponseHello> seyHello(
      $grpc.ServiceCall call, $0.RequestHello request);
}

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
