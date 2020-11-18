CFLAGS := -L../libruntime -lruntime

.PHONY: app libruntime clean clean-all
app: main.o out.o libruntime
	$(CC) -o app.app main.o out.o $(CFLAGS)

clean:
	rm -f *.o
	rm -f app.app

clean-all: clean
	rm -f out.c out.h
	cd ../libruntime && make clean

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

libruntime:
	cd ../libruntime && make
