use epaint::Vec2;

pub fn fit_size_into(inner: Vec2, container: Vec2) -> Vec2 {
    let inner_aspect = inner.x / inner.y;
    let container_aspect = container.x / container.y;

    let resize_factor =
        if container_aspect < inner_aspect { container.x / inner.x } else { container.y / inner.y };

    Vec2 { x: inner.x * resize_factor, y: inner.y * resize_factor }
}
