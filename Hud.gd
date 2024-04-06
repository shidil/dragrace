extends Node2D


# Called when the node enters the scene tree for the first time.
func _ready():
	var car = get_node("../Player")
	car.connect("metric_update", _on_car_update)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass

func _on_car_update(rpm, gear, speed):
	$rpm_label/value.text = str(rpm)
	$gear_label/value.text = str(gear)
	$speed_label/value.text = str(speed)
