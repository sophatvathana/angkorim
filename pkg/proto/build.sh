export PATH="$PATH:$(go env GOPATH)/bin"
protoc --go_out=../../../ --go-grpc_out=../../../ *.proto
protoc --dart_out=grpc:. *.proto --plugin=protoc-gen-dart=$HOME/.pub-cache/bin/protoc-gen-dart 
mv *.dart ../../client/test/bin/