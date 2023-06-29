/*
 * 版权属于：yitter(yitter@126.com)
 * 开源地址：https://github.com/yitter/idgenerator
 */
use crate::idgen::*;
use chrono::Utc;
use std::thread::sleep;

pub struct SnowWorkerM1 {
    ///基础时间
    pub base_time: i64,
    ///机器码
    pub worker_id: u32,
    ///机器码位长
    pub worker_id_bit_length: u8,
    ///自增序列数位长
    pub seq_bit_length: u8,
    ///最大序列数（含）
    pub max_seq_number: u32,
    ///最小序列数（含）
    pub min_seq_number: u32,
    ///最大漂移次数
    pub top_over_cost_count: u32,

    _timestamp_shift: u8,
    _current_seq_number: u32,
    _last_time_tick: i64,
    _turn_back_time_tick: i64,
    _turn_back_index: u8,
    _is_over_cost: bool,
    _over_cost_count_in_one_term: u32,
    _gen_count_in_one_term: u32,
    _term_index: u32,
}

impl SnowWorkerM1 {
    pub fn default() -> SnowWorkerM1 {
        let options = IdGeneratorOptions::new(1);
        return SnowWorkerM1::new(options);
    }

    pub fn set_options(&mut self, options: IdGeneratorOptions) {
        // 1.BaseTime
        if options.base_time == 0 {
            self.base_time = 1582136402000;
        } else if options.base_time < 631123200000
            || options.base_time > Utc::now().timestamp_millis()
        {
            panic!("BaseTime error.");
        } else {
            self.base_time = options.base_time;
        }

        // 2.WorkerIdBitLength
        if options.worker_id_bit_length <= 0 {
            panic!("WorkerIdBitLength error.(range:[1, 21])");
        }
        if options.seq_bit_length + options.worker_id_bit_length > 22 {
            panic!("error：WorkerIdBitLength + SeqBitLength <= 22");
        } else {
            // self.WorkerIdBitLength = options.WorkerIdBitLength;
            self.worker_id_bit_length = if options.worker_id_bit_length <= 0 {
                6
            } else {
                options.worker_id_bit_length
            };
        }

        // 3.WorkerId
        let mut max_worker_id_number = (1 << options.worker_id_bit_length) - 1;
        if max_worker_id_number == 0 {
            max_worker_id_number = 63;
        }
        if options.worker_id > max_worker_id_number {
            panic!("WorkerId error. (range:[0, {} ]", max_worker_id_number);
        } else {
            self.worker_id = options.worker_id;
        }

        // 4.SeqBitLength
        if options.seq_bit_length < 2 || options.seq_bit_length > 21 {
            panic!("SeqBitLength error. (range:[2, 21])");
        } else {
            // self.SeqBitLength = options.SeqBitLength;
            self.seq_bit_length = if options.seq_bit_length <= 0 {
                6
            } else {
                options.seq_bit_length
            };
        }

        // 5.MaxSeqNumber
        let mut max_seq_number = (1 << options.seq_bit_length) - 1;
        if options.max_seq_number <= 0 {
            max_seq_number = 63;
        }
        if options.max_seq_number > max_seq_number {
            panic!("MaxSeqNumber error. (range:[1, {}]", max_seq_number);
        } else {
            self.max_seq_number = if options.max_seq_number == 0 {
                max_seq_number
            } else {
                options.max_seq_number
            };
        }

        // 6.MinSeqNumber
        if options.min_seq_number < 5 || options.min_seq_number > max_seq_number {
            panic!("MinSeqNumber error. (range:[5, {}]", max_seq_number);
        } else {
            self.min_seq_number = options.min_seq_number;
            // self.MinSeqNumber = if options.MinSeqNumber <= 0 { 5 } else { options.MinSeqNumber };
        }

        // 7.TopOverCostCount
        //self.TopOverCostCount = if options.TopOverCostCount == 0 { 2000 } else { options.TopOverCostCount };
        if options.top_over_cost_count > 10000 {
            panic!("TopOverCostCount error. (range:[0, 10000]");
        } else {
            self.top_over_cost_count = options.top_over_cost_count;
        }

        // 8.Others
        self._timestamp_shift = self.worker_id_bit_length + self.seq_bit_length;
        self._current_seq_number = self.min_seq_number;

        if options.method == 1 {
            sleep(std::time::Duration::from_millis(500));
        }
    }

    pub fn new(options: IdGeneratorOptions) -> SnowWorkerM1 {
        let mut worker = SnowWorkerM1 {
            base_time: 1582136402000,
            worker_id_bit_length: 0,
            worker_id: 0,
            seq_bit_length: 0,
            max_seq_number: 0,
            min_seq_number: 0,
            top_over_cost_count: 0,
            _timestamp_shift: 0,
            _current_seq_number: 0,

            _last_time_tick: 0,
            _turn_back_time_tick: 0,
            _turn_back_index: 0,
            _is_over_cost: false,
            _over_cost_count_in_one_term: 0,
            _gen_count_in_one_term: 0,
            _term_index: 0,
        };

        worker.set_options(options);
        return worker;
    }

    pub fn next_id(&mut self) -> i64 {
        // println!("SeqBitLength: {}", self.SeqBitLength);
        if self._is_over_cost {
            self.next_over_cost_id()
        } else {
            self.next_normal_id()
        }
    }

    fn begin_over_cost_action(&self) {}

    fn end_over_cost_action(&mut self) {
        // if self._TermIndex > 10000 {
        //     self._TermIndex = 0;
        // }
    }

    fn begin_turn_back_action(&self) {}

    fn end_turn_back_action(&self) {}

    fn next_over_cost_id(&mut self) -> i64 {
        let current_time_tick = self.get_current_time_tick();

        if current_time_tick > self._last_time_tick {
            self.end_over_cost_action();

            self._last_time_tick = current_time_tick;
            self._current_seq_number = self.min_seq_number;
            self._is_over_cost = false;
            self._over_cost_count_in_one_term = 0;
            self._gen_count_in_one_term = 0;

            return self.calc_id(self._last_time_tick);
        }

        if self._over_cost_count_in_one_term >= self.top_over_cost_count {
            self.end_over_cost_action();

            self._last_time_tick = self.get_next_time_tick();
            self._current_seq_number = self.min_seq_number;
            self._is_over_cost = false;
            self._over_cost_count_in_one_term = 0;
            self._gen_count_in_one_term = 0;

            return self.calc_id(self._last_time_tick);
        }

        if self._current_seq_number > self.max_seq_number {
            self._last_time_tick += 1;
            self._current_seq_number = self.min_seq_number;
            self._is_over_cost = true;
            self._over_cost_count_in_one_term += 1;
            self._gen_count_in_one_term += 1;

            return self.calc_id(self._last_time_tick);
        }

        self._gen_count_in_one_term += 1;
        return self.calc_id(self._last_time_tick);
    }

    fn next_normal_id(&mut self) -> i64 {
        let current_time_tick = self.get_current_time_tick();

        if current_time_tick < self._last_time_tick {
            if self._turn_back_time_tick < 1 {
                self._turn_back_time_tick = self._last_time_tick - 1;
                self._turn_back_index += 1;
                // 每毫秒序列数的前5位是预留位，0用于手工新值，1-4是时间回拨次序
                // 支持4次回拨次序（避免回拨重叠导致ID重复），可无限次回拨（次序循环使用）。
                if self._turn_back_index > 4 {
                    self._turn_back_index = 1;
                }
                self.begin_turn_back_action();
            }

            // thread::sleep(std::time::Duration::from_millis(1));
            return self.calc_turn_back_id(self._turn_back_time_tick);
        }

        // 时间追平时，_TurnBackTimeTick清零
        if self._turn_back_time_tick > 0 {
            self.end_turn_back_action();
            self._turn_back_time_tick = 0;
        }

        if current_time_tick > self._last_time_tick {
            self._last_time_tick = current_time_tick;
            self._current_seq_number = self.min_seq_number;

            return self.calc_id(self._last_time_tick);
        }

        if self._current_seq_number > self.max_seq_number {
            self.begin_over_cost_action();

            self._term_index += 1;
            self._last_time_tick += 1;
            self._current_seq_number = self.min_seq_number;
            self._is_over_cost = true;
            self._over_cost_count_in_one_term = 1;
            self._gen_count_in_one_term = 1;

            return self.calc_id(self._last_time_tick);
        }

        return self.calc_id(self._last_time_tick);
    }

    fn calc_id(&mut self, use_time_tick: i64) -> i64 {
        let result = (use_time_tick << self._timestamp_shift)
            + (self.worker_id << self.seq_bit_length) as i64
            + (self._current_seq_number) as i64;
        self._current_seq_number += 1;
        return result;
    }

    fn calc_turn_back_id(&mut self, use_time_tick: i64) -> i64 {
        let result = (use_time_tick << self._timestamp_shift)
            + (self.worker_id << self.seq_bit_length) as i64
            + (self._turn_back_index) as i64;
        self._turn_back_time_tick -= 1;
        return result;
    }

    fn get_current_time_tick(&self) -> i64 {
        return Utc::now().timestamp_millis() - self.base_time;
    }

    fn get_next_time_tick(&self) -> i64 {
        let mut temp_time_ticker = self.get_current_time_tick();

        while temp_time_ticker <= self._last_time_tick {
            // 暂停1ms
            sleep(std::time::Duration::from_millis(1));
            temp_time_ticker = self.get_current_time_tick();
        }

        return temp_time_ticker;
    }
}
