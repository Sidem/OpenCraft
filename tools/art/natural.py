"""The stylized Ember & Aether ramps of the natural materials (docs/ART_STYLE.md sections 1-11). The natural blocks
themselves moved to the realistic look (section 12): the ground to `real.py`, the other rocks to `rocks.py`, wood to
`wood.py`, the canopies to `foliage.py`. `masonry.py`, `plants.py`, the machines and others still take their colours
from `RAMPS`.
"""
from lib import ramp

RAMPS = {
    "stone": ramp(["#0f1520", "#1e2431", "#323745", "#4b4f5e", "#666878", "#868390", "#a8a0a2", "#cbbdb6"]),
    "dirt": ramp(["#10080b", "#2a1114", "#4a231d", "#6b3a2b", "#8a5236", "#a86a42", "#c78a58"]),
    "grass": ramp(["#0c1a14", "#143022", "#1f4528", "#2f5e2c", "#46792f", "#66982f", "#8fb43a", "#c2d45e"]),
    "sand": ramp(["#2b2030", "#54403f", "#7e604f", "#a8836a", "#cfa985", "#e8cb9c", "#f6e6b8"]),
    "bark": ramp(["#0f080c", "#1f0f12", "#3b1b1a", "#5a2c22", "#78402f", "#9b573d", "#c07a55"]),
    "oak": ramp(["#2a1214", "#4a2420", "#6b3a2b", "#8c5236", "#ad6e46", "#c98d5c", "#e1b07e"]),
}
