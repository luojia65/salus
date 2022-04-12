// Copyright (c) 2021 by Rivos Inc.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

/// S-mode register definitions
use crate::exit::*;
use crate::inst;

/// Wrapper of the reason for a trap.
#[derive(Default)]
#[repr(C)]
pub struct SCause(u64);

impl SCause {
    /// Returns the GuestExit or an Error with the unknown exit cause.
    pub fn into_exit(&self) -> Result<GuestExit, u64> {
        let interrupt = self.0 & 0x8000_0000_0000_0000 != 0;
        if interrupt {
            // TODO parse interrupts
            Err(0)
        } else {
            // exception
            use SupervisorExceptionCause::*;
            // TODO handle the rest of the causes.
            match self.0 as u8 {
                inst::CAUSE_VIRTUAL_SUPERVISOR_ECALL => Ok(GuestExit::Exception(EcallVsMode)),
                inst::CAUSE_LOAD_GUEST_PAGE_FAULT => Ok(GuestExit::Exception(GuestLoadPageFault)),
                inst::CAUSE_STORE_GUEST_PAGE_FAULT => {
                    Ok(GuestExit::Exception(GuestStoreAmoPageFault))
                }
                cause => Err(cause as u64),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_interrupt() {
        let cause = SCause(0x8000_0000_0000_0001);
        assert_eq!(cause.into_exit(), Err(0)); // TODO check when ints are parsed
    }

    #[test]
    fn exception() {
        let cause = SCause(0xa);
        assert_eq!(
            cause.into_exit(),
            Ok(GuestExit::Exception(SupervisorExceptionCause::EcallVsMode))
        );
    }
}