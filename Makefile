configure:
	rm -rf build
	cmake -S . -B build -G Ninja
bld:
	cmake --build build
run:
	./build/waycast

br: bld run

all: configure bld run

install: bld
	mkdir -p ~/bin
	cp ./build/waycast ~/bin/waycast
	@echo "waycast installed to ~/bin/waycast"
	@echo "Make sure ~/bin is in your PATH"