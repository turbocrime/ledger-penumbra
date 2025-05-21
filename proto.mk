# Rules for proto generation

# Downloads Penumbra protos from the remote repository using buf
./proto/penumbra:
	# You can pin a specific version using the tag or commit, for example:
	# buf export --output ./proto buf.build/penumbra-zone/penumbra:d55e7e80eb42ecb205d88d270644c3e23e2cde36
	buf export --output ./proto buf.build/penumbra-zone/penumbra:main

# Generates source code from protos using buf and the buf.gen.yaml configuration
./app/src/protobuf: ./proto/penumbra
	buf generate

# Generates additional Rust bindings/header files using the custom generator
./app/rust/src/protobuf_h: ./app/src/protobuf
	@mkdir -p ./app/rust/src/protobuf_h
	@cd tools/proto-bindgen && cargo run

.PHONY: proto-clean
proto-clean:
	@rm -rfv ./app/rust/src/protobuf_h ./app/src/protobuf ./proto

.PHONY: proto
# Cleans generated files and regenerates everything from scratch
proto: proto-clean ./app/rust/src/protobuf_h
	make format
