TARGETS = server common client

all: fmt test clippy;

%:
	$(foreach t, $(TARGETS), (cd $(t); cargo $@);)