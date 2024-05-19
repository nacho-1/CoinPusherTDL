TARGETS = src/server src/thread_joiner src/threadpool src/

all: fmt test clippy;

%:
	$(foreach t, $(TARGETS), (cd $(t); cargo $@);)