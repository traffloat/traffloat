from save import SaveType, Writer
import assets
import sphere
import cylinder
import common_materials


def write_scenario(writer: Writer, pool: assets.Pool):
    core = writer.write(
        SaveType.Building,
        {
            "transform": {
                "position": {"x": -2.0, "y": 0.0, "z": 5.0},
                "scale": {"x": 2.0, "y": 2.0, "z": 2.0},
            },
            "appearance": {
                "label": {"type": "Custom", "value": "Core"},
                "distal": {
                    "type": "Pbr",
                    "mesh": sphere.Mesh().use(pool),
                    "material": common_materials.Glass().use(pool),
                },
                "proximal": {
                    "type": "Pbr",
                    "mesh": sphere.Mesh().use(pool),
                    "material": common_materials.Glass().use(pool),
                },
                "interior": {
                    "type": "Pbr",
                    "mesh": sphere.Mesh().use(pool),
                    "material": common_materials.Glass().use(pool),
                },
            },
        },
    )

    writer.write(
        SaveType.Facility,
        {
            "parent": core,
            "is_ambient": True,
            "inner": {},
            "appearance": {
                "label": {"type": "Custom", "value": "Ambient"},
                "distal": {"type": "Null"},
                "proximal": {"type": "Null"},
                "interior": {"type": "Null"},
            },
        },
    )

    garden = writer.write(
        SaveType.Building,
        {
            "transform": {
                "position": {"x": 2.0, "y": 0.0, "z": 5.0},
            },
            "appearance": {
                "label": {"type": "Custom", "value": "Garden"},
                "distal": {
                    "type": "Pbr",
                    "mesh": sphere.Mesh().use(pool),
                    "material": common_materials.Glass().use(pool),
                },
                "proximal": {
                    "type": "Pbr",
                    "mesh": sphere.Mesh().use(pool),
                    "material": common_materials.Glass().use(pool),
                },
                "interior": {
                    "type": "Pbr",
                    "mesh": sphere.Mesh().use(pool),
                    "material": common_materials.Glass().use(pool),
                },
            },
        },
    )

    writer.write(
        SaveType.Facility,
        {
            "parent": garden,
            "is_ambient": True,
            "inner": {},
            "appearance": {
                "label": {"type": "Custom", "value": "Ambient"},
                "distal": {"type": "Null"},
                "proximal": {"type": "Null"},
                "interior": {"type": "Null"},
            },
        },
    )

    writer.write(
        SaveType.Facility,
        {
            "parent": garden,
            "is_ambient": True,
            "inner": {
                "scale": {"x": 0.3, "y": 0.3, "z": 0.7},
            },
            "appearance": {
                "label": {"type": "Custom", "value": "Garden"},
                "distal": {
                    "type": "Pbr",
                    "mesh": cylinder.Mesh().use(pool),
                    "material": common_materials.RoughMonotone(
                        r=0.39, g=0.85, b=0.34
                    ).use(pool),
                },
                "proximal": {
                    "type": "Pbr",
                    "mesh": cylinder.Mesh().use(pool),
                    "material": common_materials.RoughMonotone(
                        r=0.39, g=0.85, b=0.34
                    ).use(pool),
                },
                "interior": {"type": "Null"},
            },
        },
    )
