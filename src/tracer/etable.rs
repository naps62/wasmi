use parity_wasm::elements::ValueType;
use specs::{
    etable::EventTableEntry,
    itable::Opcode,
    mtable::{MemoryReadSize, MemoryStoreSize, VarType},
    step::StepInfo,
};

use crate::{runner::ValueInternal, DEFAULT_VALUE_STACK_LIMIT};

use super::itable::IEntry;

pub enum RunInstructionTracePre {
    BrIfEqz {
        value: i32,
    },
    BrIfNez {
        value: i32,
    },
    BrTable {
        index: i32,
    },

    Call {
        args: Vec<ValueInternal>,
    },
    CallIndirect {
        table_idx: u32,
        type_idx: u32,
        offset: u32,
    },

    SetLocal {
        depth: u32,
        value: ValueInternal,
        vtype: ValueType,
    },
    SetGlobal {
        idx: u32,
        value: ValueInternal,
    },

    Load {
        offset: u32,
        raw_address: u32,
        effective_address: Option<u32>, // use option in case of memory out of bound
        vtype: ValueType,
        load_size: MemoryReadSize,
        mmid: u64,
    },
    Store {
        offset: u32,
        raw_address: u32,
        effective_address: Option<u32>,
        value: u64,
        vtype: ValueType,
        store_size: MemoryStoreSize,
        mmid: u64,
        pre_block_value1: Option<u64>,
        pre_block_value2: Option<u64>,
    },

    GrowMemory(i32),

    I32BinOp {
        left: i32,
        right: i32,
    },
    I32BinShiftOp {
        left: u64,
        right: u64,
    },

    I64BinOp {
        left: i64,
        right: i64,
    },

    I32Single(i32),
    I32Comp {
        left: i32,
        right: i32,
    },
    I64Comp {
        left: i64,
        right: i64,
    },

    I32WrapI64 {
        value: i64,
    },
    I64ExtendI32 {
        value: i32,
        sign: bool,
    },

    UnaryOp {
        operand: u64,
        vtype: VarType,
    },

    Drop,
    Select {
        val1: u64,
        val2: u64,
        cond: u64,
    },
}

#[derive(Debug, Clone)]
pub struct EEntry {
    pub id: u64,
    pub sp: u64,
    pub memory_size: usize,
    pub last_jump_eid: u64,
    pub inst: IEntry,
    pub step: StepInfo,
}

impl Into<EventTableEntry> for EEntry {
    fn into(self) -> EventTableEntry {
        EventTableEntry {
            eid: self.id,
            sp: (DEFAULT_VALUE_STACK_LIMIT as u64)
                .checked_sub(self.sp)
                .unwrap()
                .checked_sub(1)
                .unwrap(),
            last_jump_eid: self.last_jump_eid,
            inst: self.inst.into(),
            step_info: self.step.clone(),
            allocated_memory_pages: self.memory_size,
        }
    }
}

#[derive(Debug)]
pub struct ETable {
    eid: u64,
    pub(crate) entries: Vec<EEntry>,
}

impl Default for ETable {
    fn default() -> Self {
        Self {
            eid: 1,
            entries: vec![],
        }
    }
}

impl ETable {
    pub fn get_latest_eid(&self) -> u64 {
        self.entries.last().unwrap().id
    }

    pub fn get_entries(&self) -> &Vec<EEntry> {
        &self.entries
    }

    fn allocate_eid(&mut self) -> u64 {
        let r = self.eid;
        self.eid = r + 1;
        return r;
    }

    pub fn push(
        &mut self,
        module_instance_index: u16,
        func_index: u16,
        sp: u64,
        memory_size: usize,
        pc: u32,
        last_jump_eid: u64,
        opcode: Opcode,
        step: StepInfo,
    ) -> EEntry {
        let eentry = EEntry {
            id: self.allocate_eid(),
            sp,
            memory_size,
            last_jump_eid,
            inst: IEntry {
                module_instance_index: module_instance_index as u16,
                func_index,
                pc: pc as u16,
                opcode,
            },
            step,
        };

        self.entries.push(eentry.clone());

        eentry
    }
}
