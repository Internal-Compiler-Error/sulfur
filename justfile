diesel-redo:
    diesel migration redo && diesel setup

diesel-migrate:
    diesel migration run && diesel setup

diesel-revert:
    diesel migration revert
