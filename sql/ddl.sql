-- PostgreSQL版本的短链接服务表结构

-- 创建链接历史记录表
create table if not exists link_history
(
    id          bigint                             not null primary key,
    origin_url  varchar(4000)                      not null,
    link_type   integer                            null,
    expire_date timestamp                          null,
    active      boolean                            not null default true,
    link_hash   varchar(48)                        not null,
    create_time timestamp default CURRENT_TIMESTAMP null,
    update_time timestamp default CURRENT_TIMESTAMP null,
    constraint link_history_link_hash_uindex unique (link_hash)
);

-- 创建索引
create index if not exists link_history_link_type_index on link_history (link_type);

-- 添加表注释
comment on table link_history is '链接历史记录表';
comment on column link_history.origin_url is '原始的地址';
comment on column link_history.link_type is '链接类型 1:短期 2:长期';
comment on column link_history.active is '是否有效的';
comment on column link_history.link_hash is '链接的hash值';

-- 创建自动更新update_time的触发器函数
create or replace function update_updated_at_column()
returns trigger as $$
begin
    new.update_time = CURRENT_TIMESTAMP;
return new;
end;
$$ language plpgsql;

-- 创建触发器，在更新link_history表时自动更新update_time字段
drop trigger if exists update_link_history_updated_at on link_history;
create trigger update_link_history_updated_at
    before update on link_history
    for each row
    execute function update_updated_at_column();