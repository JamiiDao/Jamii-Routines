use pinocchio::{
    AccountView, Address, ProgramResult, entrypoint, sysvars::instructions::Instructions,
};
use solana_program_log::log;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    log("Hello from my pinocchio program!");

    let instructions: Instructions<Ref<[u8]>> = Instructions::try_from(accounts.instructions)?;
    Ok(())
}
