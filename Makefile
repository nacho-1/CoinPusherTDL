TARGETS = server common client

all: fmt test clippy;

%:
	$(foreach t, $(TARGETS), (cd $(t); cargo $@);)

run_server:
	cargo build && cargo run -p server

run_client:
	cargo build && cargo run -p client localhost 1883