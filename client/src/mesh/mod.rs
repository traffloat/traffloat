mod cube;
mod cylinder;
mod sphere;

pub struct StdMeshes {
    pub cube:           three_d::CPUMesh,
    pub cylinder:       three_d::CPUMesh,
    pub fused_cylinder: three_d::CPUMesh,
    pub sphere:         three_d::CPUMesh,
}

impl StdMeshes {
    pub fn compute() -> Self {
        Self {
            cube:           cube::compute(),
            cylinder:       cylinder::compute(false),
            fused_cylinder: cylinder::compute(true),
            sphere:         sphere::compute(),
        }
    }
}
