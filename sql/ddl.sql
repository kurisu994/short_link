create table if not exists link_history
(
    id          bigint                             not null
        primary key,
    origin_url  varchar(4000)                      not null comment '原始的地址',
    link_type   int(3)                                null comment '链接类型 1:短期 2:长期',
    expire_date datetime                           null,
    active      tinyint(1)                         not null comment '是否有效的',
    link_hash   varchar(48)                        not null comment '链接的hash值',
    create_time datetime default CURRENT_TIMESTAMP null,
    update_time datetime default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP,
    constraint link_history_link_hash_uindex
        unique (link_hash)
);

create index link_history_link_type_index
    on link_history (link_type);

