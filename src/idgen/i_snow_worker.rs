/*
 * 版权属于：yitter(yitter@126.com)
 * 开源地址：https://github.com/yitter/idgenerator
 */
#[allow(dead_code)]
pub trait ISnowWorker {
    fn next_id(&self) -> i64;
}
