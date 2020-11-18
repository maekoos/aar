.PHONY: hashset build clean libruntime builddy

libs := ./hashset
libruntime_h := $(wildcard ./*.h)
libruntime_o := $(addsuffix .o,$(basename $(filter-out $(libs), $(wildcard ./*.c))))

build: hashset libruntime
	echo $(libruntime_o)
	ar rcs libruntime.a $(libruntime_o) $(wildcard ./hashset/*.o)

libruntime: $(libruntime_o)

clean:
	cd hashset/ && $(MAKE) clean
	rm -f *.a
	rm -f *.o

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

hashset:
	cd hashset/ && $(MAKE) hashset.o
	cd hashset/ && $(MAKE) hashset_itr.o