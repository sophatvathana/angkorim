import 'dart:ffi';
import 'dart:io' show WebSocket, sleep;
import 'dart:convert' show json;
import 'dart:async' show Timer;
import 'dart:typed_data';

import 'package:protobuf/protobuf.dart';

import 'angkorim.pb.dart';

main() {
  for (var i = 0; i < 600; i++) {
    print("connecting $i");
    WebSocket.connect('ws://0.0.0.0:9523/').then((WebSocket ws) {
      if (ws.readyState == WebSocket.open) {
        print("connected $i");
        SignInRequest req = SignInRequest();
        req.phoneNumber = "85596604088$i";
        req.deviceId = "device-b";
        ws.add(encode(Command.CMD_SIGNIN, req));
        ws.add(
            encode(Command.CMD_SUBSCRIBE_TOPIC, SubscribeRequest(topic: "A")));
        // ws.add(encode(Command.CMD_SEND_MSG,SimpleMessageRequest(topic: "A", message: "Hello world")));
        ws.listen(
          (data) {
            Response res = Response.fromBuffer(data);
            switch (res.cmd) {
              case Command.CMD_SIGNIN:
                if (res.code == ResponseCode.REQUEST_SUCCESS) {
                  print("signIn error code:${res.code};message:${res.message}");
                }
                SignInResponse signInResponse =
                    SignInResponse.fromBuffer(res.data);
                print(signInResponse.token);
                break;
              case Command.CMD_SEND_MSG:
                SimpleMessageResponse response =
                    SimpleMessageResponse.fromBuffer(res.data);
                print(response);
                break;
              default:
            }
          },
          onDone: () => print('[+]Done :)'),
          onError: (err) => print('[!]Error -- ${err.toString()}'),
          cancelOnError: true,
        );
      } else
        print('[!]Connection Denied');
      // in case, if serer is not running now
    }, onError: (err) => print('[!]Error -- ${err.toString()}'));
    sleep(Duration(milliseconds: 10));
  }
}

Uint8List encode(Command command, GeneratedMessage message) {
  var request = Request();
  request.cmd = command;
  // if (requestId == null) {
  //   request.requestId = Int64(DateTime.now().microsecondsSinceEpoch);
  // }
  if (message != null) {
    request.data = message.writeToBuffer();
  }

  var buffer = request.writeToBuffer();
  return buffer;
}
