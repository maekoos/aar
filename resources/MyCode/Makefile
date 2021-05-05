.PHONY: compile to_class to_dex dexdump clean run-java
compile: dexdump

run-java: to_class
	java MyCode

to_class: *.java
	javac ./*.java 

to_dex: to_class
	d8 ./*.class

dexdump: to_dex
	dexdump -d ./*.dex > classes.dedexed

clean:
	rm -f ./*.class ./*.dex ./*.dedexed