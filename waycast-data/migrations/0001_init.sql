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
create index if not exists idx_items_staging_item_id_kind on items(item_id,kind);