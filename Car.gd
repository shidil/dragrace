extends Node2D


var speed_max = 400 # kmph
var gear = 1
var throttle = 1
var engine_health = 100

# 2023 mustang GT TORSEN
var rpm = 800
var rpm_max = 7800
var rev_limitter = 7500
var gear_ratios = [3.237, 2.104, 1.422, 1.0, 0.814, 0.622]
var final_drive = 3.73
var tyre = Vector3(235,55,17)
var wheel_radi = calc_wheel_radi(tyre.x, tyre.y, tyre.z)

signal metric_update(rpm, gear, speed)
	
# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	if throttle > 0.1:
		rpm = min(rev_limitter, rpm + (throttle * 1000 * delta))
		
	if rpm >= rev_limitter:
		if rpm > rpm_max:
			engine_health = engine_health - 0.1
		else:
			engine_health = engine_health - 0.01
		print_debug("engine took damage::", engine_health)

	var speed = calc_speed(rpm, wheel_radi, gear_ratios[gear - 1], final_drive)
	position.x += speed * delta
	
	# updates hud
	metric_update.emit(snapped(rpm, 1), gear, snapped(speed, 1))
	
func shift_up():
	var max_gear = len(gear_ratios)
	if gear == max_gear:
		return
		
	var current_ratio = gear_ratios[gear - 1]
	var next_ratio = gear_ratios[gear]
	var wheel_rpm = rpm / (current_ratio * final_drive)
	
	gear = gear + 1
	rpm = wheel_rpm * (next_ratio * final_drive)

func calc_speed(rpm, wheel_radi, gear_ratio, final_drive): # kmph
	# vv [m/s]=vw [m/s]= Ne⋅π⋅rw / 30⋅ix⋅i0
	var engine_speed = (rpm * PI) / 30
	var wheel_speed = engine_speed / (gear_ratio * final_drive)
	var mps = wheel_speed * wheel_radi
	return mps * 3.6

func calc_wheel_radi(w, ar, d):
	var sidewall_height = (w * ar) / 100
	var wheel_diameter = (d * 25.4) + (2 * sidewall_height)
	return wheel_diameter / 1000 / 2
