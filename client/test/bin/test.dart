import 'dart:ffi';
import 'dart:io' show WebSocket, stdin, stdout;
import 'dart:convert' show json;
import 'dart:async' show Timer;
import 'dart:typed_data';

import 'package:protobuf/protobuf.dart';

import 'angkorim.pb.dart';

Future<WebSocket> connect() async {
  var ws = await WebSocket.connect('ws://0.0.0.0:9522/api/ws');
  if (ws.readyState == WebSocket.open) {
    SignInRequest req = SignInRequest();
    req.phoneNumber = "85599987498";
    req.deviceId = "device-A";
    ws.add(encode(Command.CMD_SIGNIN,req));  
    ws.add(encode(Command.CMD_SUBSCRIBE_TOPIC,SubscribeRequest(topic: "A")));  
    // ws.add(encode(Command.CMD_SEND_MSG,SimpleMessageRequest(topic: "A", message: "Hello world"))); 
    ws.listen(  
      (data) {
        Response res = Response.fromBuffer(data);
        switch (res.cmd) {
          case Command.CMD_SIGNIN:
            if(res.code == ResponseCode.REQUEST_SUCCESS){
              print("signIn error code:${res.code};message:${res.message}");
            }
            SignInResponse signInResponse = SignInResponse.fromBuffer(res.data);
            print(signInResponse.token);
            break;
          case Command.CMD_SEND_MSG:
            SimpleMessageResponse response = SimpleMessageResponse.fromBuffer(res.data);
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
  return ws;
}

main() {
  connect().then((ws) {
    while(ws.readyState == WebSocket.open) {
      stdout.write("Enter your text : ");
      var text = stdin.readLineSync();
      ws.add(encode(Command.CMD_SEND_MSG,SimpleMessageRequest(topic: "A", message: text))); 
    }
  });
  
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