use nalgebra::Vector3;

#[inline(always)]
pub fn lerp<T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Mul<f32, Output = T> + Copy>(a: T, b: T, t: f32) -> T {
    a + (b - a) * t
}

#[inline(always)]
pub fn is_greater_than_zero(v: Vector3<f32>) -> bool {
    v.x > 0.0 || v.y > 0.0 || v.z > 0.0
}