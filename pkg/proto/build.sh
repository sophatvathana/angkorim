protoc --go_out=plugins=grpc:../../../ *.proto
protoc --dart_out=grpc:. *.proto --plugin=protoc-gen-dart=$HOME/.pub-cache/bin/protoc-gen-dart 
mv *.dart ../../client/test/bin/