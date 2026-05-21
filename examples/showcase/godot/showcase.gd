# This file is a small runnable Godot-side smoke sample for the generated config.

extends Node

const CONFIG_PATH := "res://config/config.json"

func _ready() -> void:
	var config := SoraConfig.load_json_file(CONFIG_PATH)
	var items := config.item()
	var sword = items.get_row(1001)
	print("loaded %d items; first item=%s" % [items.length(), sword.name])
