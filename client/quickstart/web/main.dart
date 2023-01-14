import 'dart:html';
import 'dart:typed_data';

import 'angkorim.pb.dart';
import 'package:protobuf/protobuf.dart';

var websocket = WebSocket("ws://0.0.0.0:9523/")..binaryType = 'arraybuffer';
void main() {
  querySelector("#selectUsername")?.onClick.listen((event) {
    print("username!");
    var username = (querySelector('#username') as InputElement).value;
    print(username);
    SignInRequest req = SignInRequest();
    req.phoneNumber = username!;
    req.deviceId = "device-A";
    websocket.send(encode(Command.CMD_SIGNIN, req));
    websocket.send(
        encode(Command.CMD_SUBSCRIBE_TOPIC, SubscribeRequest(topic: "A")));
  });
  websocket.onMessage.listen((event) {
    ByteBuffer buf = event.data;
    Response res = Response.fromBuffer(buf.asUint8List());
    switch (res.cmd) {
      case Command.CMD_SIGNIN:
        if (res.code == ResponseCode.REQUEST_SUCCESS) {
          print("signIn error code:${res.code};message:${res.message}");
        }
        querySelector(".setuser")?.remove();
        querySelector(".dim")?.remove();
        SignInResponse signInResponse = SignInResponse.fromBuffer(res.data);
        print(signInResponse.token);
        break;
      case Command.CMD_SEND_MSG:
        SimpleMessageResponse response =
            SimpleMessageResponse.fromBuffer(res.data);

        print("Response: ${response.message}\n");
        var html = """<div class="youmessage"> 
			 <div class="avatar"></div> 
			 <div class="name"></div> 
			 <div class="text">${response.message}</div> 
		    </div>""";
        querySelector(".chatbox .messages-wrapper")?.append(Element.html(html));
        // querySelector(".chatbox")
        //     ?.scrollTop(querySelector(".chatbox .messages-wrapper").height());
        break;
      default:
    }
  });

  querySelector(".send")?.onClick.listen((event) {
    websocket.send(encode(
        Command.CMD_SEND_MSG,
        SimpleMessageRequest(
            topic: "A",
            message: (querySelector('.usermsg') as InputElement).value)));
    (querySelector('.usermsg') as InputElement).value = "";
  });
  // websocket.send(data);
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
