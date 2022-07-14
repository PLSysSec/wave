
bin: fuzz.c
	gcc $< -g -o bin

fuzz-bin: fuzz.c
	afl-gcc $< -o fuzz-bin

fuzz: fuzz-bin
	rm -rf ramdisk/*
	if [[ -e ramdisk ]]; then\
		sudo umount ./ramdisk;\
		rmdir ramdisk;\
	fi
	mkdir ramdisk
	sudo mount -t tmpfs -o size=1024M tmpfs ./ramdisk/
	cp -r testcase-dir ./ramdisk/testcase-dir
	mkdir ./ramdisk/findings-dir
	AFL_NO_FORKSRV=1 AFL_SKIP_CPUFREQ=1 afl-fuzz -i ramdisk/testcase-dir -o ramdisk/findings-dir -m 16000 ./$< @@

clean:
	rm -rf ramdisk/*
	sudo umount ./ramdisk
	rmdir ramdisk

check-crashes: fuzz-bin
	bash check_crashes.sh
