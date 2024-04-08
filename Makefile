build:
	cd rust && cargo build

run: build
	cd godot && godot4

open:
	godot4 godot/project.godot > /dev/null 2>&1 &disown
