use crate::vertex::ClipSpaceVertex;

pub trait Clipper {
    // 接收一个裁剪空间的三角形
    // 返回一个 Vec，其中包含裁剪后产生的零个、一个或多个三角形
    fn clip_triangle(&self, triangle: &[ClipSpaceVertex; 3]) -> Vec<[ClipSpaceVertex; 3]>;
}


// 这是一个只实现了简单“丢弃”逻辑的裁剪器
pub struct SimpleClipper;

impl Clipper for SimpleClipper {
    fn clip_triangle(&self, triangle: &[ClipSpaceVertex; 3]) -> Vec<[ClipSpaceVertex; 3]> {
        let v0_w = triangle[0].position.w;
        let v1_w = triangle[1].position.w;
        let v2_w = triangle[2].position.w;

        // 简单的近平面裁剪：如果所有顶点都在相机后面，则丢弃
        if v0_w < 0.0 && v1_w < 0.0 && v2_w < 0.0 {
            // 返回一个空 Vec，表示这个三角形被完全裁剪掉了
            vec![]
        } else {
            // 否则，暂时保留整个三角形。
            // 这已经解决了性能问题，虽然在视觉上还不完美。
            vec![*triangle]
        }
    }
}