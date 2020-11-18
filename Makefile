.PHONY: setup-example clean dex-parser-gen compile-MyCode
setup-and-run-example: dex-parser-gen compile-MyCode
	RUST_LOG=aar=info cargo run

dex-parser-gen:
	cd src/parser/dex-parser-gen && make

compile-MyCode:
	cd resources/MyCode/ && make

clean:
	cd out/ && make clean-all
	cd resources/MyCode/ && make clean

clean-hard: clean
	cargo clean