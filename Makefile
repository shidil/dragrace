build:
	cd rust && cargo build

run: build
	cd godot && godot4
