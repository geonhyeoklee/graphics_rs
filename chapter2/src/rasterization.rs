use glam::{Vec2, Vec3};
use std::vec::Vec;

#[derive(Clone, Debug)]
pub struct MyVertex {
  pos: Vec3,
  color: Vec3,
}

#[derive(Clone, Debug)]
pub struct MyTriangle {
  v0: MyVertex,
  v1: MyVertex,
  v2: MyVertex,
}

pub struct Rasterization {
  width: i32,
  height: i32,
  triangle: MyTriangle,
}

impl Rasterization {
  pub fn new(width: i32, height: i32) -> Self {
    let triangle = MyTriangle {
      v0: MyVertex {
        pos: Vec3::new(0.0, 0.0, 0.0),
        color: Vec3::new(1.0, 0.0, 0.0),
      },
      v1: MyVertex {
        pos: Vec3::new(1.0, 0.0, 0.0),
        color: Vec3::new(0.0, 1.0, 0.0),
      },
      v2: MyVertex {
        pos: Vec3::new(0.0, 1.0, 0.0),
        color: Vec3::new(0.0, 0.0, 1.0),
      },
    };

    Self {
      width,
      height,
      triangle,
    }
  }

  pub fn project_world_to_raster(&self, point: Vec3) -> Vec2 {
    // 종횡비(aspect ratio) 계산
    let aspect = self.width as f32 / self.height as f32;

    // 종횡비를 고려하여 NDC 좌표계로 변환
    let point_ndc = Vec2::new(point.x / aspect, point.y);

    // 래스터 변환을 위한 스케일 계수
    let x_scale = 2.0 / self.width as f32;
    let y_scale = 2.0 / self.height as f32;

    // y축 방향을 뒤집어서 래스터 좌표계로 변환
    Vec2::new(
      (point_ndc.x + 1.0) / x_scale - 0.5,
      (1.0 - point_ndc.y) / y_scale - 0.5,
    )
  }

  pub fn edge_function(&self, v0: Vec2, v1: Vec2, point: Vec2) -> f32 {
    (point.x - v0.x) * (v1.y - v0.y) - (point.y - v0.y) * (v1.x - v0.x)
  }

  pub fn render(&self) -> Vec<[f32; 4]> {
    let mut pixels = vec![[0.0; 4]; (self.width * self.height) as usize];

    // 정점들을 래스터 공간으로 투영
    let v0 = self.project_world_to_raster(self.triangle.v0.pos);
    let v1 = self.project_world_to_raster(self.triangle.v1.pos);
    let v2 = self.project_world_to_raster(self.triangle.v2.pos);

    // 경계 상자(bounding box) 찾기
    let x_min = v0.x.min(v1.x).min(v2.x).max(0.0) as i32;
    let x_max = v0.x.max(v1.x).max(v2.x).min(self.width as f32 - 1.0) as i32;
    let y_min = v0.y.min(v1.y).min(v2.y).max(0.0) as i32;
    let y_max = v0.y.max(v1.y).max(v2.y).min(self.height as f32 - 1.0) as i32;

    // 경계 상자 내의 각 픽셀에 대해 반복
    for j in y_min..=y_max {
      for i in x_min..=x_max {
        let point = Vec2::new(i as f32, j as f32);

        // 중심 좌표(barycentric coordinates) 계산
        let alpha0 = self.edge_function(v1, v2, point);
        let alpha1 = self.edge_function(v2, v0, point);
        let alpha2 = self.edge_function(v0, v1, point);

        // point가 삼각형 내부에 있을 때, 다시 말해 alpha0, alpha1, alpha2가 모두 양수인지 확인
        if alpha0 >= 0.0 && alpha1 >= 0.0 && alpha2 >= 0.0 {
          // 중심 좌표 정규화
          let total = alpha0 + alpha1 + alpha2;

          let (alpha0, alpha1, alpha2) = if total != 0.0 {
            (alpha0 / total, alpha1 / total, alpha2 / total)
          } else {
            (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
          };

          // 색상 보간
          let color = self.triangle.v0.color * alpha0
            + self.triangle.v1.color * alpha1
            + self.triangle.v2.color * alpha2;

          let idx = (i + j * self.width) as usize;
          pixels[idx] = [color.x, color.y, color.z, 1.0];
        }
      }
    }

    pixels
  }

  pub fn update(&mut self) {
    // Implement update logic here
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new_rasterization() {
    let raster = Rasterization::new(800, 600);
    assert_eq!(raster.width, 800);
    assert_eq!(raster.height, 600);
  }

  #[test]
  fn test_project_world_to_raster() {
    let raster = Rasterization::new(800, 600);
    let world_point = Vec3::new(0.5, 0.5, 0.0);
    let raster_point = raster.project_world_to_raster(world_point);

    // 예상되는 래스터 좌표 계산
    let aspect = 800.0 / 600.0;
    let expected_x = ((0.5 / aspect + 1.0) / (2.0 / 800.0)) - 0.5;
    let expected_y = ((1.0 - 0.5) / (2.0 / 600.0)) - 0.5;

    assert_eq!(raster_point.x, expected_x);
    assert_eq!(raster_point.y, expected_y);
  }

  #[test]
  fn test_edge_function() {
    let raster = Rasterization::new(800, 600);
    let v0 = Vec2::new(0.0, 0.0);
    let v1 = Vec2::new(1.0, 0.0);
    let point_inside = Vec2::new(0.5, -0.1);
    let point_outside = Vec2::new(0.5, 0.1);

    // 점이 선분의 왼쪽에 있으면 양수, 오른쪽에 있으면 음수
    assert!(raster.edge_function(v0, v1, point_inside) > 0.0);
    assert!(raster.edge_function(v0, v1, point_outside) < 0.0);
  }

  #[test]
  fn test_render_output() {
    let raster = Rasterization::new(4, 4);
    let pixels = raster.render();

    // 출력 버퍼의 크기 확인
    assert_eq!(pixels.len(), 16);

    // 모든 픽셀이 유효한 값을 가지는지 확인
    for pixel in pixels.iter() {
      assert!(pixel[0] >= 0.0 && pixel[0] <= 1.0);
      assert!(pixel[1] >= 0.0 && pixel[1] <= 1.0);
      assert!(pixel[2] >= 0.0 && pixel[2] <= 1.0);
      // assert_eq!(pixel[3], 1.0);
    }
  }
}
