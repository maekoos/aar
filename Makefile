.PHONY: setup-and-run-example dex-parser-gen compile-MyCode run-java compile-out run debug clean clean-hard

setup-and-run-example: dex-parser-gen compile-MyCode run

dex-parser-gen:
	cd src/parser/dex-parser-gen && make

compile-MyCode:
	cd resources/MyCode/ && make

run-java:
	cd resources/MyCode/ && make run-java

compile-out:
	cd out/ && make

run:
	RUST_LOG=aar=info cargo run

debug:
	RUST_LOG=aar=debug cargo run

clean:
	rm -r out/
	cd resources/MyCode/ && make clean

clean-hard: clean
	cargo clean