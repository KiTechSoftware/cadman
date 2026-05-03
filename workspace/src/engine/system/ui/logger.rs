impl super::Ui {
    pub fn heading(&self, msg: &str) {
        self.logger().heading(msg);
    }

    pub fn info(&self, msg: &str) {
        self.logger().info(msg);
    }

    pub fn warn(&self, msg: &str) {
        self.logger().warn(msg);
    }

    pub fn error(&self, msg: &str) {
        self.logger().error(msg);
    }

    pub fn error_kv(&self, key: &str, value: &str) {
        self.logger().error_kv(key, value);
    }

    pub fn debug(&self, msg: &str) {
        self.logger().debug(msg);
    }

    pub fn trace(&self, msg: &str) {
        self.logger().trace(msg);
    }

    pub fn kv(&self, key: &str, value: &str) {
        self.logger().kv(key, value);
    }
}
