use nalgebra::Vector3;

pub fn reflect(a: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
    a - a.dot(&n) * 2.0 * n
}

pub fn refract(i: Vector3<f32>, n: Vector3<f32>, ni: f32, nr: f32) -> Vector3<f32> {
    let cos_i = -i.dot(&n);
    if cos_i.is_sign_negative() {
        return refract(i, -n, nr, ni);
    }
    let eta = ni / nr;
    let cos_r_sq = 1.0 - eta * eta * (1.0 - cos_i * cos_i);
    if cos_r_sq.is_sign_negative() {
        // total reflection
        -reflect(i, n)
    } else {
        i * eta + n * (eta * cos_i - f32::sqrt(cos_r_sq))
    }
}
