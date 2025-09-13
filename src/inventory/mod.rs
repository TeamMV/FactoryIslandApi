use crate::ingredients::IngredientStack;

pub struct InventoryData {
    stacks: Vec<IngredientStack>,
    current_amt: u64,
    item_limit: u64,
}

impl InventoryData {
    pub fn new(limit: u64) -> Self {
        Self {
            stacks: Vec::new(),
            current_amt: 0,
            item_limit: limit
        }
    }

    /// WARNING 😱🚨: This function will consume the stack, whether it is added or not. Do not let it eat stacks it cannot handle!
    pub fn add_stack(&mut self, other_stack: IngredientStack) {
        if self.can_handle(&other_stack) {
            self.current_amt += other_stack.amount;
            for stack in &mut self.stacks {
                if stack.is_mergeable(&other_stack) {
                    stack.amount += other_stack.amount;
                    return;
                }
            }
            self.stacks.push(other_stack);
        }
    }

    pub fn can_handle(&self, other_stack: &IngredientStack) -> bool {
        if let Some(amount) = self.current_amt.checked_add(other_stack.amount) { amount <= self.item_limit } else { false }
    }
}