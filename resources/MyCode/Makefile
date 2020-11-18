compile: dexdump

to_class: *.java
	javac ./*.java 

to_dex: to_class
	d8 ./*.class

dexdump: to_dex
	dexdump -d ./*.dex > classes.dedexed

clean:
	rm -f ./*.class ./*.dex ./*.dedexed