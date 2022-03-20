
.PHONY: clean_all
clean_all: ./*
	@for file in $^ ; do \
		make -C $${file} clean ; \
	done
