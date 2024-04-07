extends Node2D

var Car = load("res://CarPhysics.gd")
var car

# 2023 mustang GT TORSEN
@export var idle_rpm = 800
@export var rpm_max = 7800
@export var rev_limitter = 7500
@export var gear_ratios = [3.237, 2.104, 1.422, 1.0, 0.814, 0.622]
@export var final_drive = 3.73
@export var tyre = Vector3(235,55,17)
@export var weight = 1692 # Kg
@export var torque_curve = [0, 275, 400, 450, 500, 525, 550, 575, 575, 575, 550, 525, 500, 475, 400]

signal metric_update(rpm, gear, speed)

enum GearBox {SHIFTING, SHIFT_COMPLETE, SHIFT_FAILED}
enum CarState {IDLE, MOVING, STALLED, BUSTED}

var car_state: CarState
var gearbox_state: GearBox

func reset():
	car = Car.new(gear_ratios, final_drive, weight, tyre)
	car.start()
	position.x = 0
	
# Called when the node enters the scene tree for the first time.
func _ready():
	reset()

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	if Input.is_action_pressed("shift_up"):
		car.shift_up()
	if Input.is_action_pressed("reset_game"):
		reset()
	
	car.update(delta)
	position.x += 1
	
	# updates hud
	#metric_update.emit(snapped(rpm, 1), gear, snapped(to_kpmph(speed), 1))
	
func to_kpmph(mps):
	return mps * 3.6
