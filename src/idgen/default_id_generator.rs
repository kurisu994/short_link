/*
 * 版权属于：yitter(yitter@126.com)
 * 开源地址：https://github.com/yitter/idgenerator
 */
use crate::idgen::*;

pub struct DefaultIdGenerator {
    pub worker: SnowWorkerM1,
}

impl DefaultIdGenerator {
    pub fn default() -> DefaultIdGenerator {
        DefaultIdGenerator {
            worker: SnowWorkerM1::default(),
        }
    }
}
