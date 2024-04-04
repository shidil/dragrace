extends Node2D


var acceleration = 0
var speed = 0
var power = 0
var gear = 1
var throttle = 1
var engine_health = 100
var drag = 0.1

# 2023 mustang GT TORSEN
var idle_rpm = 800
var rpm = idle_rpm
var rpm_max = 7800
var rev_limitter = 7500
var speed_max = 400 # m/s
var gear_ratios = [3.237, 2.104, 1.422, 1.0, 0.814, 0.622]
var final_drive = 3.73
var tyre = Vector3(235,55,17)
var max_power = 246 # KW
var weight = 1692 # Kg
var wheel_radi = calc_wheel_radi(tyre.x, tyre.y, tyre.z)

signal metric_update(rpm, gear, speed)

enum GearBox {SHIFTING, SHIFT_COMPLETE, SHIFT_FAILED}
enum CarState {IDLE, MOVING, STALLED, BUSTED}

var car_state: CarState
var gearbox_state: GearBox

func reset():
	rpm = idle_rpm
	speed = 0
	acceleration = 0
	power = max_power
	gear = 1
	throttle = 1
	engine_health = 100
	position.x = 0
	car_state = CarState.MOVING
	gearbox_state = GearBox.SHIFT_COMPLETE
	
# Called when the node enters the scene tree for the first time.
func _ready():
	#$SfxrStreamPlayer2D.play()
	reset()

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	if Input.is_action_pressed("shift_up"):
		shift_up()
	if Input.is_action_pressed("reset_game"):
		reset()
		
	var gear_ratio = gear_ratios[gear - 1]
	if car_state != CarState.MOVING or gearbox_state == GearBox.SHIFTING:
		power = 0
		rpm = calc_rpm(speed, wheel_radi, gear_ratio, final_drive)
	if car_state == CarState.MOVING and gearbox_state != GearBox.SHIFTING:
		if throttle > 0.1:
			rpm = min(rev_limitter, rpm + (throttle * 10))
			power = max_power
			print_debug("power  ", power)
	# rpm, and speed cannot be negative
	rpm = max(0, rpm)
	speed = max(0, speed)
	
		# stall happens to manual transmission
	if rpm < idle_rpm - 100:
		car_state = CarState.STALLED

	if rpm >= rev_limitter:
		if rpm > rpm_max:
			engine_health = engine_health - 0.1
		else:
			engine_health = engine_health - 0.01
		print_debug("engine took damage::", engine_health)
		
	var prev_speed = speed
	var v = calc_speed(rpm, wheel_radi, gear_ratio, final_drive)
	prev_speed = min(prev_speed, v)
	drag = prev_speed / 10000
	prev_speed = prev_speed - drag
	var force = (power / max(prev_speed, 1)) * 1000 # N
	acceleration = force / weight
	position.x += prev_speed * delta + (0.5 * acceleration * delta * delta)
	speed = prev_speed + (acceleration * delta)
	
	# updates hud
	metric_update.emit(snapped(rpm, 1), gear, snapped(to_kpmph(speed), 1))
	
func shift_up():
	var max_gear = len(gear_ratios)
	if gear == max_gear:
		return
		
	if gearbox_state == GearBox.SHIFTING:
		return
		
	gearbox_state = GearBox.SHIFTING
	$ShiftTimer.start(0.3)
		
	var current_ratio = gear_ratios[gear - 1]
	var next_ratio = gear_ratios[gear]
	var wheel_rpm = rpm / (current_ratio * final_drive)
	
	gear = gear + 1
	rpm = wheel_rpm * (next_ratio * final_drive)

func calc_speed(rpm, wheel_radi, gear_ratio, final_drive): # m/s
	# vv [m/s]=vw [m/s]= Ne⋅π⋅rw / 30⋅ix⋅i0
	var engine_speed = (rpm * PI) / 30
	var wheel_speed = engine_speed / (gear_ratio * final_drive)
	var mps = wheel_speed * wheel_radi
	return mps

func calc_wheel_radi(w, ar, d):
	var sidewall_height = (w * ar) / 100
	var wheel_diameter = (d * 25.4) + (2 * sidewall_height)
	return wheel_diameter / 1000 / 2
	
func calc_rpm(mps, wheel_radi, gear_ratio, final_drive):
	var wheel_speed = mps / wheel_radi
	var engine_speed = wheel_speed * (gear_ratio * final_drive)
	var engine_rpm = (engine_speed * 30) / PI
	return engine_rpm
	
func to_kpmph(mps):
	return mps * 3.6

func _on_shift_timer_timeout():
	gearbox_state = GearBox.SHIFT_COMPLETE
