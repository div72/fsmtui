#[derive(Copy, Clone, PartialEq)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

impl Vector2D {
    pub fn rotate(&self, angle: f64) -> Self {
        let mut angle = angle % std::f64::consts::TAU;
        if angle < 0.0 {
            angle += std::f64::consts::TAU;
        }

        let (sin, cos) = angle.sin_cos();
        Self {
            x: cos * self.x - sin * self.y,
            y: sin * self.x + cos * self.y,
        }
    }

    pub fn magnitude(&self) -> f64 {
        self.x.hypot(self.y)
    }

    pub fn normalized(&self) -> Self {
        let magnitude = self.magnitude();

        if magnitude == 0.0 {
            *self
        } else {
            *self / magnitude
        }
    }
}

impl std::ops::Add for Vector2D {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Vector2D {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Mul<f64> for Vector2D {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl std::ops::Div<f64> for Vector2D {
    type Output = Self;

    fn div(self, rhs: f64) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl From<Vector2D> for (f64, f64) {
    fn from(v: Vector2D) -> Self {
        (v.x, v.y)
    }
}
