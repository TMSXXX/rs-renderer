use crate::framebuffer::FrameBuffer;
use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};
use cgmath::Vector4 as Vec4;
use rand::{Rng, random_bool};

pub fn glitch_effect(framebuffer: &mut FrameBuffer) {
    let width = WINDOW_WIDTH;
    let height = WINDOW_HEIGHT;
    let total_pixels = width * height;

    // 克隆原始数据用于读取
    let original_data = framebuffer.data.clone();
    if original_data.len() != total_pixels {
        return;
    }

    let mut rng = rand::rng();

    // 遍历所有行
    for y in 0..height {
        // 15% 的概率对当前行应用 glitch 效果
        if rng.random_bool(0.1) {
            // 水平偏移量（限制在 [-8, 8]）
            let offset = rng.random_range(-8..8);
            let affect_rows = rng.random_range(1..=10);

            // 随机选择行内的一个水平区间 [start_x, end_x)
            let start_x = rng.random_range(0..width);
            // 区间宽度随机（10到宽度的1/3）
            let segment_width = rng.random_range(10..=width / 4);
            let end_x = (start_x + segment_width).min(width);
            let reverse = rng.random_bool(0.3);

            // 处理连续的几行
            for dy in 0..affect_rows {
                let current_y = y + dy;
                if current_y >= height {
                    break;
                }

                // 只处理选中的水平区间内的像素
                for x in start_x..end_x {
                    // 计算源像素 x 坐标（应用偏移）
                    let src_x = (x as i32 + offset).clamp(0, width as i32 - 1) as usize;

                    let dst_idx = current_y * width + x;
                    let src_idx = current_y * width + src_x;

                    // 随机修改颜色通道
                    let mut color = original_data[src_idx];
                    if reverse {
                        // 随机交换颜色通道
                        // 反转颜色（假设颜色值在 [0.0, 1.0] 范围内）
                        color.x = 1.0 - color.x;
                        color.y = 1.0 - color.y;
                        color.z = 1.0 - color.z;
                    } else if random_bool(0.3) {
                        match rng.random_range(0..3) {
                            0 => std::mem::swap(&mut color.x, &mut color.y),
                            1 => std::mem::swap(&mut color.x, &mut color.z),
                            2 => std::mem::swap(&mut color.y, &mut color.z),
                            _ => {}
                        }
                    }  else if rng.random_bool(0.2) {
                        // 反转颜色（假设颜色值在 [0.0, 1.0] 范围内）
                        color.x = 1.0 - color.x;
                        color.y = 1.0 - color.y;
                        color.z = 1.0 - color.z;
                    }

                    framebuffer.data[dst_idx] = color;
                }
            }
        } else {
            // 对不应用行偏移的行，随机扰动个别像素
            for x in 0..width {
                if rng.random_bool(0.02) {
                    let rand_x =
                        (x as i32 + rng.random_range(-3..3)).clamp(0, width as i32 - 1) as usize;
                    let rand_y =
                        (y as i32 + rng.random_range(-1..1)).clamp(0, height as i32 - 1) as usize;
                    let src_idx = rand_y * width + rand_x;
                    framebuffer.data[y * width + x] = original_data[src_idx];
                }
            }
        }
    }
}
