configure:
	rm -rf build
	cmake -S . -B build -G Ninja
bld:
	cmake --build build
run:
	./build/waycast

br: bld run

all: configure bld run