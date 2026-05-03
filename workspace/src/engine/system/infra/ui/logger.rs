
impl super::Ui {
    fn heading(&self, msg: &str) {
        self.logger().heading(msg);
    }

    fn info(&self, msg: &str) {
        self.logger().info(msg);
    }

    fn warn(&self, msg: &str) {
        self.logger().warn(msg);
    }

    fn error(&self, msg: &str) {
        self.logger().error(msg);
    }

    fn error_kv(&self, key: &str, value: &str) {
        self.logger().error_kv(key, value);
    }

    fn debug(&self, msg: &str) {
        self.logger().debug(msg);
    }

    fn trace(&self, msg: &str) {
        self.logger().trace(msg);
    }

    fn kv(&self, key: &str, value: &str) {
        self.logger().kv(key, value);
    }
}
