extends Node

@export var runningSound: AudioStreamOggVorbis
var player: AudioStreamPlayer2D

# Called when the node enters the scene tree for the first time.
func _ready():
	runningSound.loop = true
	runningSound.loop_offset = 1
	player = $RunningAudio
	player.stream = runningSound
	player.play()
	pass # Replace with function body.

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass

func _on_player_metric_update(rpm, gear, speed):
	var pitch_scale = max(0.5, (rpm / 7500.0) * 2)
	player.pitch_scale = lerpf(player.pitch_scale, pitch_scale, get_process_delta_time())
