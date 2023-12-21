use super::super::ColMatrix;
use super::{
    chiplets::{
        hasher::{DIGEST_LEN, STATE_WIDTH},
        BITWISE_A_COL_IDX, BITWISE_B_COL_IDX, BITWISE_OUTPUT_COL_IDX, HASHER_NODE_INDEX_COL_IDX,
        HASHER_STATE_COL_RANGE, MEMORY_ADDR_COL_IDX, MEMORY_CLK_COL_IDX, MEMORY_CTX_COL_IDX,
        MEMORY_V_COL_RANGE,
    },
    decoder::{
        GROUP_COUNT_COL_IDX, HASHER_STATE_OFFSET, IN_SPAN_COL_IDX, IS_CALL_FLAG_COL_IDX,
        IS_LOOP_BODY_FLAG_COL_IDX, IS_LOOP_FLAG_COL_IDX, IS_SYSCALL_FLAG_COL_IDX,
        NUM_HASHER_COLUMNS, NUM_OP_BATCH_FLAGS, OP_BATCH_FLAGS_OFFSET, OP_BITS_EXTRA_COLS_OFFSET,
        USER_OP_HELPERS_OFFSET,
    },
    stack::{B0_COL_IDX, B1_COL_IDX, H0_COL_IDX},
    CHIPLETS_OFFSET, CLK_COL_IDX, CTX_COL_IDX, DECODER_TRACE_OFFSET, FMP_COL_IDX, FN_HASH_OFFSET,
    STACK_TRACE_OFFSET,
};
use core::ops::Range;
use vm_core::{utils::range, Felt, ONE, ZERO};

// CONSTANTS
// ================================================================================================

const DECODER_HASHER_RANGE: Range<usize> =
    range(DECODER_TRACE_OFFSET + HASHER_STATE_OFFSET, NUM_HASHER_COLUMNS);

// HELPER STRUCT AND METHODS
// ================================================================================================

pub struct MainTrace<'a> {
    columns: &'a ColMatrix<Felt>,
}

impl<'a> MainTrace<'a> {
    pub fn new(main_trace: &'a ColMatrix<Felt>) -> Self {
        Self {
            columns: main_trace,
        }
    }

    // SYSTEM COLUMNS
    // --------------------------------------------------------------------------------------------

    /// Returns the value of the clk column at row i.
    pub fn clk(&self, i: usize) -> Felt {
        self.columns.get_column(CLK_COL_IDX)[i]
    }

    /// Returns the value of the fmp column at row i.
    pub fn fmp(&self, i: usize) -> Felt {
        self.columns.get_column(FMP_COL_IDX)[i]
    }

    /// Returns the value of the ctx column at row i.
    pub fn ctx(&self, i: usize) -> Felt {
        self.columns.get_column(CTX_COL_IDX)[i]
    }

    // DECODER COLUMNS
    // --------------------------------------------------------------------------------------------

    /// Returns the value in the block address column at the row i.
    pub fn addr(&self, i: usize) -> Felt {
        self.columns.get_column(DECODER_TRACE_OFFSET)[i]
    }

    /// Helper method to detect change of address.
    pub fn is_addr_change(&self, i: usize) -> bool {
        self.addr(i) != self.addr(i + 1)
    }

    /// First decoder helper register at row i.
    pub fn helper_0(&self, i: usize) -> Felt {
        self.columns.get_column(DECODER_TRACE_OFFSET + USER_OP_HELPERS_OFFSET)[i]
    }

    /// Second decoder helper register at row i.
    pub fn helper_1(&self, i: usize) -> Felt {
        self.columns.get_column(DECODER_TRACE_OFFSET + USER_OP_HELPERS_OFFSET + 1)[i]
    }

    /// Third decoder helper register at row i.
    pub fn helper_2(&self, i: usize) -> Felt {
        self.columns.get_column(DECODER_TRACE_OFFSET + USER_OP_HELPERS_OFFSET + 2)[i]
    }

    /// Returns the hasher state at row i.
    pub fn decoder_hasher_state(&self, i: usize) -> [Felt; NUM_HASHER_COLUMNS] {
        let mut state = [ZERO; NUM_HASHER_COLUMNS];
        for (idx, col_idx) in DECODER_HASHER_RANGE.enumerate() {
            let column = self.columns.get_column(col_idx);
            state[idx] = column[i];
        }
        state
    }

    /// Returns the first half of the hasher state at row i.
    pub fn decoder_hasher_state_first_half(&self, i: usize) -> [Felt; DIGEST_LEN] {
        let mut state = [ZERO; DIGEST_LEN];
        for (col, s) in state.iter_mut().enumerate() {
            *s = self.columns.get_column(DECODER_TRACE_OFFSET + HASHER_STATE_OFFSET + col)[i];
        }
        state
    }

    /// Returns a specific element from the hasher state at row i.
    pub fn decoder_hasher_state_element(&self, element: usize, i: usize) -> Felt {
        self.columns.get_column(DECODER_TRACE_OFFSET + HASHER_STATE_OFFSET + element)[i + 1]
    }

    /// Returns the current function hash (i.e., root) at row i.
    pub fn fn_hash(&self, i: usize) -> [Felt; DIGEST_LEN] {
        let mut state = [ZERO; DIGEST_LEN];
        for (col, s) in state.iter_mut().enumerate() {
            *s = self.columns.get_column(FN_HASH_OFFSET + col)[i];
        }
        state
    }

    /// Returns the `is_loop_body` flag at row i.
    pub fn is_loop_body_flag(&self, i: usize) -> Felt {
        self.columns.get_column(DECODER_TRACE_OFFSET + IS_LOOP_BODY_FLAG_COL_IDX)[i]
    }

    /// Returns the `is_loop` flag at row i.
    pub fn is_loop_flag(&self, i: usize) -> Felt {
        self.columns.get_column(DECODER_TRACE_OFFSET + IS_LOOP_FLAG_COL_IDX)[i]
    }

    /// Returns the `is_call` flag at row i.
    pub fn is_call_flag(&self, i: usize) -> Felt {
        self.columns.get_column(DECODER_TRACE_OFFSET + IS_CALL_FLAG_COL_IDX)[i]
    }

    /// Returns the `is_syscall` flag at row i.
    pub fn is_syscall_flag(&self, i: usize) -> Felt {
        self.columns.get_column(DECODER_TRACE_OFFSET + IS_SYSCALL_FLAG_COL_IDX)[i]
    }

    /// Returns the operation batch flags at row i. This indicates the number of op groups in
    /// the current batch that is being processed.
    pub fn op_batch_flag(&self, i: usize) -> [Felt; NUM_OP_BATCH_FLAGS] {
        [
            self.columns.get(DECODER_TRACE_OFFSET + OP_BATCH_FLAGS_OFFSET, i),
            self.columns.get(DECODER_TRACE_OFFSET + OP_BATCH_FLAGS_OFFSET + 1, i),
            self.columns.get(DECODER_TRACE_OFFSET + OP_BATCH_FLAGS_OFFSET + 2, i),
        ]
    }

    /// Returns the operation group count. This indicates the number of operation that remain
    /// to be executed in the current span block.
    pub fn group_count(&self, i: usize) -> Felt {
        self.columns.get_column(DECODER_TRACE_OFFSET + GROUP_COUNT_COL_IDX)[i]
    }

    /// Returns the delta between the current and next group counts.
    pub fn delta_group_count(&self, i: usize) -> Felt {
        self.group_count(i) - self.group_count(i + 1)
    }

    /// Returns the `in_span` flag at row i.
    pub fn is_in_span(&self, i: usize) -> Felt {
        self.columns.get_column(DECODER_TRACE_OFFSET + IN_SPAN_COL_IDX)[i]
    }

    /// Constructs the i-th op code value from its individual bits.
    pub fn get_op_code(&self, i: usize) -> Felt {
        let col_b0 = self.columns.get_column(DECODER_TRACE_OFFSET + 1);
        let col_b1 = self.columns.get_column(DECODER_TRACE_OFFSET + 2);
        let col_b2 = self.columns.get_column(DECODER_TRACE_OFFSET + 3);
        let col_b3 = self.columns.get_column(DECODER_TRACE_OFFSET + 4);
        let col_b4 = self.columns.get_column(DECODER_TRACE_OFFSET + 5);
        let col_b5 = self.columns.get_column(DECODER_TRACE_OFFSET + 6);
        let col_b6 = self.columns.get_column(DECODER_TRACE_OFFSET + 7);
        let [b0, b1, b2, b3, b4, b5, b6] =
            [col_b0[i], col_b1[i], col_b2[i], col_b3[i], col_b4[i], col_b5[i], col_b6[i]];
        b0 + b1.mul_small(2)
            + b2.mul_small(4)
            + b3.mul_small(8)
            + b4.mul_small(16)
            + b5.mul_small(32)
            + b6.mul_small(64)
    }

    /// Returns a flag indicating whether the current operation induces a left shift of the operand
    /// stack.
    pub fn is_left_shift(&self, i: usize) -> bool {
        let b0 = self.columns.get(DECODER_TRACE_OFFSET + 1, i);
        let b1 = self.columns.get(DECODER_TRACE_OFFSET + 2, i);
        let b2 = self.columns.get(DECODER_TRACE_OFFSET + 3, i);
        let b3 = self.columns.get(DECODER_TRACE_OFFSET + 4, i);
        let b4 = self.columns.get(DECODER_TRACE_OFFSET + 5, i);
        let b5 = self.columns.get(DECODER_TRACE_OFFSET + 6, i);
        let b6 = self.columns.get(DECODER_TRACE_OFFSET + 7, i);
        let e0 = self.columns.get(DECODER_TRACE_OFFSET + OP_BITS_EXTRA_COLS_OFFSET, i);
        let h5 = self.columns.get(DECODER_TRACE_OFFSET + IS_LOOP_FLAG_COL_IDX, i);

        // group with left shift effect grouped by a common prefix
        ([b6, b5, b4] == [ZERO, ONE, ZERO])||
        // U32ADD3 or U32MADD
        ([b6, b5, b4, b3, b2] == [ONE, ZERO, ZERO, ONE, ONE]) ||
        // SPLIT or LOOP block
        ([e0, b3, b2, b1] == [ONE, ZERO, ONE, ZERO]) ||
        // REPEAT
        ([b6, b5, b4, b3, b2, b1, b0] == [ONE, ONE, ONE, ZERO, ONE, ZERO, ZERO]) ||
        // END of a loop
        ([b6, b5, b4, b3, b2, b1, b0] == [ONE, ONE, ONE, ZERO, ZERO, ZERO, ZERO] && h5 == ONE)
    }

    /// Returns a flag indicating whether the current operation induces a right shift of the operand
    /// stack.
    pub fn is_right_shift(&self, i: usize) -> bool {
        let b0 = self.columns.get(DECODER_TRACE_OFFSET + 1, i);
        let b1 = self.columns.get(DECODER_TRACE_OFFSET + 2, i);
        let b2 = self.columns.get(DECODER_TRACE_OFFSET + 3, i);
        let b3 = self.columns.get(DECODER_TRACE_OFFSET + 4, i);
        let b4 = self.columns.get(DECODER_TRACE_OFFSET + 5, i);
        let b5 = self.columns.get(DECODER_TRACE_OFFSET + 6, i);
        let b6 = self.columns.get(DECODER_TRACE_OFFSET + 7, i);

        // group with right shift effect grouped by a common prefix
        [b6, b5, b4] == [ZERO, ONE, ONE]||
        // u32SPLIT 100_1000
        ([b6, b5, b4, b3, b2, b1, b0] == [ONE, ZERO, ZERO, ONE, ZERO, ZERO, ZERO]) ||
        // PUSH i.e., 110_0100
        ([b6, b5, b4, b3, b2, b1, b0] == [ONE, ONE, ZERO, ZERO, ONE, ZERO, ZERO])
    }

    // STACK COLUMNS
    // --------------------------------------------------------------------------------------------

    /// Returns the value of the stack depth column at row i.
    pub fn stack_depth(&self, i: usize) -> Felt {
        self.columns.get_column(STACK_TRACE_OFFSET + B0_COL_IDX)[i]
    }

    /// Returns the element at row i in a given stack trace column.
    pub fn stack_element(&self, column: usize, i: usize) -> Felt {
        self.columns.get_column(STACK_TRACE_OFFSET + column)[i]
    }

    /// Returns the address of the top element in the stack overflow table at row i.
    pub fn parent_overflow_address(&self, i: usize) -> Felt {
        self.columns.get_column(STACK_TRACE_OFFSET + B1_COL_IDX)[i]
    }

    /// Returns a flag indicating whether the overflow stack is non-empty.
    pub fn is_non_empty_overflow(&self, i: usize) -> bool {
        let b0 = self.columns.get_column(STACK_TRACE_OFFSET + B0_COL_IDX)[i];
        let h0 = self.columns.get_column(STACK_TRACE_OFFSET + H0_COL_IDX)[i];
        (b0 - Felt::new(16)) * h0 == ONE
    }

    // CHIPLETS COLUMNS
    // --------------------------------------------------------------------------------------------

    /// Returns chiplet column number 0 at row i.
    pub fn chiplet_selector_0(&self, i: usize) -> Felt {
        self.columns.get_column(CHIPLETS_OFFSET)[i]
    }

    /// Returns chiplet column number 1 at row i.
    pub fn chiplet_selector_1(&self, i: usize) -> Felt {
        self.columns.get_column(CHIPLETS_OFFSET + 1)[i]
    }

    /// Returns chiplet column number 2 at row i.
    pub fn chiplet_selector_2(&self, i: usize) -> Felt {
        self.columns.get_column(CHIPLETS_OFFSET + 2)[i]
    }

    /// Returns chiplet column number 3 at row i.
    pub fn chiplet_selector_3(&self, i: usize) -> Felt {
        self.columns.get_column(CHIPLETS_OFFSET + 3)[i]
    }

    /// Returns chiplet column number 4 at row i.
    pub fn chiplet_selector_4(&self, i: usize) -> Felt {
        self.columns.get_column(CHIPLETS_OFFSET + 4)[i]
    }

    /// Returns the (full) state of the hasher chiplet at row i.
    pub fn chiplet_hasher_state(&self, i: usize) -> [Felt; STATE_WIDTH] {
        let mut state = [ZERO; STATE_WIDTH];
        for (idx, col_idx) in HASHER_STATE_COL_RANGE.enumerate() {
            let column = self.columns.get_column(col_idx);
            state[idx] = column[i];
        }
        state
    }

    /// Returns the hasher's node index column at row i
    pub fn chiplet_node_index(&self, i: usize) -> Felt {
        self.columns.get(HASHER_NODE_INDEX_COL_IDX, i)
    }

    /// Returns the bitwise column holding the aggregated value of input `a` at row i.
    pub fn chiplet_bitwise_a(&self, i: usize) -> Felt {
        self.columns.get_column(BITWISE_A_COL_IDX)[i]
    }

    /// Returns the bitwise column holding the aggregated value of input `b` at row i.
    pub fn chiplet_bitwise_b(&self, i: usize) -> Felt {
        self.columns.get_column(BITWISE_B_COL_IDX)[i]
    }

    /// Returns the bitwise column holding the aggregated value of the output at row i.
    pub fn chiplet_bitwise_z(&self, i: usize) -> Felt {
        self.columns.get_column(BITWISE_OUTPUT_COL_IDX)[i]
    }

    /// Returns the i-th row of the chiplet column containing memory context.
    pub fn chiplet_memory_ctx(&self, i: usize) -> Felt {
        self.columns.get_column(MEMORY_CTX_COL_IDX)[i]
    }

    /// Returns the i-th row of the chiplet column containing memory address.
    pub fn chiplet_memory_addr(&self, i: usize) -> Felt {
        self.columns.get_column(MEMORY_ADDR_COL_IDX)[i]
    }

    /// Returns the i-th row of the chiplet column containing clock cycle.
    pub fn chiplet_memory_clk(&self, i: usize) -> Felt {
        self.columns.get_column(MEMORY_CLK_COL_IDX)[i]
    }

    /// Returns the i-th row of the chiplet column containing the zeroth memory value element.
    pub fn chiplet_memory_value_0(&self, i: usize) -> Felt {
        self.columns.get_column(MEMORY_V_COL_RANGE.start)[i]
    }

    /// Returns the i-th row of the chiplet column containing the first memory value element.
    pub fn chiplet_memory_value_1(&self, i: usize) -> Felt {
        self.columns.get_column(MEMORY_V_COL_RANGE.start + 1)[i]
    }

    /// Returns the i-th row of the chiplet column containing the second memory value element.
    pub fn chiplet_memory_value_2(&self, i: usize) -> Felt {
        self.columns.get_column(MEMORY_V_COL_RANGE.start + 2)[i]
    }

    /// Returns the i-th row of the chiplet column containing the third memory value element.
    pub fn chiplet_memory_value_3(&self, i: usize) -> Felt {
        self.columns.get_column(MEMORY_V_COL_RANGE.start + 3)[i]
    }

    /// Returns the i-th row of the chiplet column containing the zeroth element of the kernel
    /// procedure root.
    pub fn chiplet_kernel_root_0(&self, i: usize) -> Felt {
        self.columns.get_column(CHIPLETS_OFFSET + 6)[i]
    }

    /// Returns the i-th row of the chiplet column containing the first element of the kernel
    /// procedure root.
    pub fn chiplet_kernel_root_1(&self, i: usize) -> Felt {
        self.columns.get_column(CHIPLETS_OFFSET + 7)[i]
    }

    /// Returns the i-th row of the chiplet column containing the second element of the kernel
    /// procedure root.
    pub fn chiplet_kernel_root_2(&self, i: usize) -> Felt {
        self.columns.get_column(CHIPLETS_OFFSET + 8)[i]
    }

    /// Returns the i-th row of the chiplet column containing the third element of the kernel
    /// procedure root.
    pub fn chiplet_kernel_root_3(&self, i: usize) -> Felt {
        self.columns.get_column(CHIPLETS_OFFSET + 9)[i]
    }

    /// Returns `true` if a row is part of the kernel chiplet.
    pub fn is_kernel_row(&self, i: usize) -> bool {
        self.chiplet_selector_0(i) == ONE
            && self.chiplet_selector_1(i) == ONE
            && self.chiplet_selector_2(i) == ONE
            && self.chiplet_selector_3(i) == ZERO
    }

    //  MERKLE PATH HASHING SELECTORS
    // --------------------------------------------------------------------------------------------

    /// Returns `true` if the hasher chiplet flags indicate the initialization of verifying
    /// a Merkle path to an old node during Merkle root update procedure (MRUPDATE).
    pub fn f_mv(&self, i: usize) -> bool {
        (i % 8 == 0)
            && self.chiplet_selector_0(i) == ZERO
            && self.chiplet_selector_1(i) == ONE
            && self.chiplet_selector_2(i) == ONE
            && self.chiplet_selector_3(i) == ZERO
    }

    /// Returns `true` if the hasher chiplet flags indicate the continuation of verifying
    /// a Merkle path to an old node during Merkle root update procedure (MRUPDATE).
    pub fn f_mva(&self, i: usize) -> bool {
        (i % 8 == 7)
            && self.chiplet_selector_0(i) == ZERO
            && self.chiplet_selector_1(i) == ONE
            && self.chiplet_selector_2(i) == ONE
            && self.chiplet_selector_3(i) == ZERO
    }

    /// Returns `true` if the hasher chiplet flags indicate the initialization of verifying
    /// a Merkle path to a new node during Merkle root update procedure (MRUPDATE).
    pub fn f_mu(&self, i: usize) -> bool {
        (i % 8 == 0)
            && self.chiplet_selector_0(i) == ZERO
            && self.chiplet_selector_1(i) == ONE
            && self.chiplet_selector_2(i) == ONE
            && self.chiplet_selector_3(i) == ONE
    }

    /// Returns `true` if the hasher chiplet flags indicate the continuation of verifying
    /// a Merkle path to a new node during Merkle root update procedure (MRUPDATE).
    pub fn f_mua(&self, i: usize) -> bool {
        (i % 8 == 7)
            && self.chiplet_selector_0(i) == ZERO
            && self.chiplet_selector_1(i) == ONE
            && self.chiplet_selector_2(i) == ONE
            && self.chiplet_selector_3(i) == ONE
    }
}
