/*
 * 版权属于：yitter(yitter@126.com)
 * 开源地址：https://github.com/yitter/idgenerator
 */
use crate::idgen::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;

pub struct YitIdHelper;

static ID_GEN_INSTANCE: OnceLock<Arc<Mutex<DefaultIdGenerator>>> = OnceLock::new();

impl YitIdHelper {
    fn id_gen_instance() -> Arc<Mutex<DefaultIdGenerator>> {
        ID_GEN_INSTANCE
            .get_or_init(|| Arc::new(Mutex::new(DefaultIdGenerator::default())))
            .clone()
    }

    #[allow(dead_code)]
    pub fn set_id_generator(options: IdGeneratorOptions) {
        let idgen_arc = YitIdHelper::id_gen_instance();
        let mut idgen = idgen_arc.lock().unwrap();
        idgen.worker.set_options(options);
    }

    #[allow(dead_code)]
    pub fn set_worker_id(worker_id: u32) {
        let idgen_arc = YitIdHelper::id_gen_instance();
        let mut idgen = idgen_arc.lock().unwrap();
        let options = IdGeneratorOptions::new(worker_id);
        idgen.worker.set_options(options);
    }

    #[allow(dead_code)]
    pub fn next_id() -> i64 {
        let idgen_arc = YitIdHelper::id_gen_instance();
        let mut idgen = idgen_arc.lock().unwrap();
        idgen.worker.next_id()
    }
}
