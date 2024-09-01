from save import SaveType
import models
import sphere


def write_scenario(writer):
    writer.write(
        SaveType.Building,
        {
            "transform": {
                "position": {"x": 0.0, "y": 0.0, "z": 5.0},
            },
            "appearance": {
                "distal": {
                    "type": "Pbr",
                    "mesh": models.register_gltf_mesh(sphere.mesh),
                    "material": models.register_material(sphere.material_glass),
                },
                "proximal": {
                    "type": "Pbr",
                    "mesh": models.register_gltf_mesh(sphere.mesh),
                    "material": models.register_material(sphere.material_glass),
                },
                "interior": {
                    "type": "Pbr",
                    "mesh": models.register_gltf_mesh(sphere.mesh),
                    "material": models.register_material(sphere.material_glass),
                },
            },
        },
    )
