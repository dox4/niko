-- Add migration script here
CREATE TABLE
  entry (
    `id` INT PRIMARY KEY AUTO_INCREMENT,
    `parent` VARCHAR(1000) NOT NULL,
    `name` VARCHAR(100) NOT NULL,
    `is_dir` TINYINT NOT NULL,
    `size` INT NOT NULL,
    `permission` INT NOT NULL,
    `created_at` DATETIME NOT NULL,
    `updated_at` DATETIME NOT NULL,
    `deleted_at` DATETIME NULL
  ) ENGINE = InnoDB AUTO_INCREMENT = 112774 DEFAULT CHARSET = utf8mb4;
