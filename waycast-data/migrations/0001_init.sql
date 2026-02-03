create table if not exists items (
    id integer primary key,
    item_id text not null,
    kind text not null,
    title text not null,
    description text,
    icon text not null,

    unique(item_id, kind)
);

create index if not exists idx_items_kind on items(kind);
create index if not exists idx_items_item_id_kind on items(item_id,kind);

-- This table will be truncated and re-filled
-- every time we do a new scan. Then we
-- will do a diff with the main "items"
-- table and delete whatever records
-- are not found in staging based
-- on the item_id and kind combo.
create table if not exists items_staging (
    id integer primary key,
    item_id text not null,
    kind text not null,
    title text not null,
    description text,
    icon text not null,

    unique(item_id, kind)
);

create index if not exists idx_items_staging_kind on items_staging(kind);
create index if not exists idx_items_staging_item_id_kind on items_staging(item_id,kind);

-- Search index
CREATE VIRTUAL TABLE IF NOT EXISTS items_fts USING fts5(
  title,
  description,
  content='items',
  content_rowid='id'
);

CREATE TRIGGER IF NOT EXISTS items_ai AFTER INSERT ON items BEGIN
  INSERT INTO items_fts(rowid, title, description)
  VALUES (new.id, new.title, new.description);
END;

CREATE TRIGGER IF NOT EXISTS items_ad AFTER DELETE ON items BEGIN
  delete from items_fts
  where items_fts.rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS items_au AFTER UPDATE ON items BEGIN
  delete from items_fts
  where items_fts.rowid = old.id;

  INSERT INTO items_fts(rowid, title, description)
  VALUES (new.id, new.title, new.description);
END;

-- Icons table
create table if not exists icons (
    id integer primary key,
    name text not null,
    path text not null,

    unique(name)
);

-- Cache table
create table if not exists cache (
    key text primary key not null,
    value text not null,
    expires_at integer null,

    unique(key)
);