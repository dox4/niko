-- Add migration script here
CREATE TABLE
  metadata (
    `key` VARCHAR(64) NOT NULL,
    `value` VARCHAR(64) NOT NULL,
    `updated_at` DATETIME NOT NULL,
    UNIQUE(`key`)
  ) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4;
