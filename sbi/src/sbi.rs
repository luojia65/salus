// Copyright (c) 2021 by Rivos Inc.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

//! Rust SBI message parsing.
//! `SbiMessage` is an enum of all the SBI extensions.
//! For each extension, a function enum is defined to contain the SBI function data.
#![no_std]

mod consts;
pub use consts::*;
mod error;
pub use error::*;
mod function;
pub use function::*;
// The TSM SBI extension
mod tsm;
pub use tsm::*;

/// Interfaces for invoking SBI functionality.
pub mod api;

#[cfg(all(target_arch = "riscv64", target_os = "none"))]
use core::arch::asm;

/// Functions defined for the Base extension
#[derive(Clone, Copy)]
pub enum BaseFunction {
    /// Returns the implemented version of the SBI standard.
    GetSpecificationVersion,
    /// Returns the ID of the SBI implementation.
    GetImplementationID,
    /// Returns the version of this SBI implementation.
    GetImplementationVersion,
    /// Checks if the given SBI extension is supported.
    ProbeSbiExtension(u64),
    /// Returns the vendor that produced this machine(`mvendorid`).
    GetMachineVendorID,
    /// Returns the architecture implementation ID this machine(`marchid`).
    GetMachineArchitectureID,
    /// Returns the implementation ID of this machine(`mimpid`).
    GetMachineImplementationID,
}

impl BaseFunction {
    /// Attempts to parse `Self` from the passed in `a0-a7`.
    fn from_regs(args: &[u64]) -> Result<Self> {
        use BaseFunction::*;

        match args[6] {
            0 => Ok(GetSpecificationVersion),
            1 => Ok(GetImplementationID),
            2 => Ok(GetImplementationVersion),
            3 => Ok(ProbeSbiExtension(args[0])),
            4 => Ok(GetMachineVendorID),
            5 => Ok(GetMachineArchitectureID),
            6 => Ok(GetMachineImplementationID),
            _ => Err(Error::NotSupported),
        }
    }
}

impl SbiFunction for BaseFunction {
    fn a6(&self) -> u64 {
        use BaseFunction::*;
        match self {
            GetSpecificationVersion => 0,
            GetImplementationID => 1,
            GetImplementationVersion => 2,
            ProbeSbiExtension(_) => 3,
            GetMachineVendorID => 4,
            GetMachineArchitectureID => 5,
            GetMachineImplementationID => 6,
        }
    }

    fn a0(&self) -> u64 {
        use BaseFunction::*;
        match self {
            ProbeSbiExtension(ext) => *ext,
            _ => 0,
        }
    }
}

/// Functions defined for the State extension
#[derive(Clone, Copy)]
pub enum StateFunction {
    /// Starts the given hart.
    HartStart {
        /// a0 - hart id to start.
        hart_id: u64,
        /// a1 - address to start the hart.
        start_addr: u64,
        /// a2 - value to be set in a1 when starting the hart.
        opaque: u64,
    },
    /// Stops the current hart.
    HartStop,
    /// Returns the status of the given hart.
    HartStatus {
        /// a0 - ID of the hart to check.
        hart_id: u64,
    },
    /// Requests that the calling hart be suspended.
    HartSuspend {
        /// a0 - Specifies the type of suspend to initiate.
        suspend_type: u32,
        /// a1 - The address to jump to on resume.
        resume_addr: u64,
        /// a2 - An opaque value to load in a1 when resuming the hart.
        opaque: u64,
    },
}

/// Return value for the HartStatus SBI call.
#[repr(u64)]
pub enum HartState {
    /// The hart is physically powered-up and executing normally.
    Started = 0,
    /// The hart is not executing in supervisor-mode or any lower privilege mode.
    Stopped = 1,
    /// Some other hart has requested to start, operation still in progress.
    StartPending = 2,
    /// Some other hart has requested to stop, operation still in progress.
    StopPending = 3,
    /// This hart is in a platform specific suspend (or low power) state.
    Suspended = 4,
    /// The hart has requested to put itself in a platform specific low power state, in progress.
    SuspendPending = 5,
    /// An interrupt or platform specific hardware event has caused the hart to resume normal
    /// execution. Resuming is ongoing.
    ResumePending = 6,
}

impl StateFunction {
    /// Attempts to parse `Self` from the passed in `a0-a7`.
    fn from_regs(args: &[u64]) -> Result<Self> {
        use StateFunction::*;
        match args[6] {
            0 => Ok(HartStart {
                hart_id: args[0],
                start_addr: args[1],
                opaque: args[2],
            }),
            1 => Ok(HartStop),
            2 => Ok(HartStatus { hart_id: args[0] }),
            3 => Ok(HartSuspend {
                suspend_type: args[0] as u32,
                resume_addr: args[1],
                opaque: args[2],
            }),
            _ => Err(Error::NotSupported),
        }
    }
}

impl SbiFunction for StateFunction {
    fn a6(&self) -> u64 {
        use StateFunction::*;
        match self {
            HartStart { .. } => 0,
            HartStop => 1,
            HartStatus { .. } => 2,
            HartSuspend { .. } => 3,
        }
    }

    fn a0(&self) -> u64 {
        use StateFunction::*;
        match self {
            HartStart {
                hart_id,
                start_addr: _,
                opaque: _,
            } => *hart_id,
            HartStatus { hart_id } => *hart_id,
            HartSuspend {
                suspend_type,
                resume_addr: _,
                opaque: _,
            } => *suspend_type as u64,
            _ => 0,
        }
    }

    fn a1(&self) -> u64 {
        use StateFunction::*;
        match self {
            HartStart {
                hart_id: _,
                start_addr,
                opaque: _,
            } => *start_addr,
            HartSuspend {
                suspend_type: _,
                resume_addr,
                opaque: _,
            } => *resume_addr,
            _ => 0,
        }
    }

    fn a2(&self) -> u64 {
        use StateFunction::*;
        match self {
            HartStart {
                hart_id: _,
                start_addr: _,
                opaque,
            } => *opaque,
            HartSuspend {
                suspend_type: _,
                resume_addr: _,
                opaque,
            } => *opaque,
            _ => 0,
        }
    }
}

/// Functions for the Reset extension
#[derive(Copy, Clone)]
pub enum ResetFunction {
    /// Performs a system reset.
    Reset {
        /// Determines the type of reset to perform.
        reset_type: ResetType,
        /// Represents the reason for system reset.
        reason: ResetReason,
    },
}

/// The types of reset a supervisor can request.
#[repr(u64)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResetType {
    /// Powers down the system.
    Shutdown = 0,
    /// Powers down, then reboots.
    ColdReset = 1,
    /// Reboots, doesn't power down.
    WarmReset = 2,
}

impl ResetType {
    // Creates a reset type from the a0 register value or returns an error if no mapping is
    // known for the given value.
    fn from_reg(a0: u64) -> Result<Self> {
        use ResetType::*;
        Ok(match a0 {
            0 => Shutdown,
            1 => ColdReset,
            2 => WarmReset,
            _ => return Err(Error::InvalidParam),
        })
    }
}

/// Reasons why a supervisor requests a reset.
#[repr(u64)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResetReason {
    /// Used for normal resets.
    NoReason = 0,
    /// Used when the system has failed.
    SystemFailure = 1,
}

impl ResetReason {
    // Creates a reset reason from the a1 register value or returns an error if no mapping is
    // known for the given value.
    fn from_reg(a1: u64) -> Result<Self> {
        use ResetReason::*;
        Ok(match a1 {
            0 => NoReason,
            2 => SystemFailure,
            _ => return Err(Error::InvalidParam),
        })
    }
}

impl ResetFunction {
    /// Creates an operation to shutdown the machine.
    pub fn shutdown() -> Self {
        ResetFunction::Reset {
            reset_type: ResetType::Shutdown,
            reason: ResetReason::NoReason,
        }
    }
}

impl ResetFunction {
    /// Attempts to parse `Self` from the passed in `a0-a7`.
    fn from_regs(args: &[u64]) -> Result<Self> {
        use ResetFunction::*;

        Ok(match args[6] {
            0 => Reset {
                reset_type: ResetType::from_reg(args[0])?,
                reason: ResetReason::from_reg(args[1])?,
            },
            _ => return Err(Error::NotSupported),
        })
    }
}

impl SbiFunction for ResetFunction {
    fn a0(&self) -> u64 {
        match self {
            ResetFunction::Reset {
                reset_type: _,
                reason,
            } => *reason as u64,
        }
    }

    fn a1(&self) -> u64 {
        match self {
            ResetFunction::Reset {
                reset_type,
                reason: _,
            } => *reset_type as u64,
        }
    }
}

/// Functions provided by the attestation extension.
#[derive(Copy, Clone)]
pub enum AttestationFunction {
    /// Get an attestion evidence from a CSR (https://datatracker.ietf.org/doc/html/rfc2986).
    /// The caller passes the CSR and its length through the first 2 arguments.
    /// The third argument is the address where the generated certificate will be placed.
    /// The evidence is formatted an x.509 DiceTcbInfo certificate extension
    ///
    /// a6 = 0
    /// a0 = CSR address
    /// a1 = CSR length
    /// a2 = Generated certificate address
    /// a3 = Reserved length for the generated certificate address
    GetEvidence {
        /// a0 = CSR address
        csr_addr: u64,
        /// a1 = CSR length
        csr_len: u64,
        /// a2 = Generated Certificate address
        cert_addr: u64,
        /// a3 = Reserved length for the generated certificate address
        cert_len: u64,
    },

    /// Extend the TCB measurement with an additional measurement.
    /// TBD: Do we allow for a specific PCR index to be passed, or do we extend
    /// one dedicated PCR with all runtime extended measurements?
    ///
    /// a6 = 0
    /// a0 = Measurement entry address
    /// a1 = Measurement entry length
    ExtendMeasurement {
        /// a0 = measurement address
        measurement_addr: u64,
        /// a1 = measurement length
        len: u64,
    },
}

impl AttestationFunction {
    /// Attempts to parse `Self` from the passed in `a0-a7`.
    pub fn from_regs(args: &[u64]) -> Result<Self> {
        use AttestationFunction::*;
        match args[6] {
            0 => Ok(GetEvidence {
                csr_addr: args[0],
                csr_len: args[1],
                cert_addr: args[2],
                cert_len: args[3],
            }),

            1 => Ok(ExtendMeasurement {
                measurement_addr: args[0],
                len: args[1],
            }),

            _ => Err(Error::InvalidParam),
        }
    }

    fn a6(&self) -> u64 {
        use AttestationFunction::*;
        match self {
            GetEvidence {
                csr_addr: _,
                csr_len: _,
                cert_addr: _,
                cert_len: _,
            } => 0,

            ExtendMeasurement {
                measurement_addr: _,
                len: _,
            } => 1,
        }
    }

    fn a3(&self) -> u64 {
        use AttestationFunction::*;
        match self {
            GetEvidence {
                csr_addr: _,
                csr_len: _,
                cert_addr: _,
                cert_len,
            } => *cert_len,

            ExtendMeasurement {
                measurement_addr: _,
                len,
            } => *len,
        }
    }

    fn a2(&self) -> u64 {
        use AttestationFunction::*;
        match self {
            GetEvidence {
                csr_addr: _,
                csr_len: _,
                cert_addr,
                cert_len: _,
            } => *cert_addr,

            ExtendMeasurement {
                measurement_addr: _,
                len,
            } => *len,
        }
    }

    fn a1(&self) -> u64 {
        use AttestationFunction::*;
        match self {
            GetEvidence {
                csr_addr: _,
                csr_len,
                cert_addr: _,
                cert_len: _,
            } => *csr_len,

            ExtendMeasurement {
                measurement_addr: _,
                len,
            } => *len,
        }
    }

    fn a0(&self) -> u64 {
        use AttestationFunction::*;
        match self {
            GetEvidence {
                csr_addr,
                csr_len: _,
                cert_addr: _,
                cert_len: _,
            } => *csr_addr,

            ExtendMeasurement {
                measurement_addr,
                len: _,
            } => *measurement_addr,
        }
    }
}

/// The values returned from an SBI function call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SbiReturn {
    /// The error code (0 for success).
    pub error_code: i64,
    /// The return value if the operation is successful.
    pub return_value: u64,
}

impl SbiReturn {
    /// Returns an `SbiReturn` that indicates success.
    pub fn success(return_value: u64) -> Self {
        Self {
            error_code: SBI_SUCCESS,
            return_value,
        }
    }
}

impl From<Result<u64>> for SbiReturn {
    fn from(result: Result<u64>) -> SbiReturn {
        match result {
            Ok(rv) => Self::success(rv),
            Err(e) => Self::from(e),
        }
    }
}

impl From<Error> for SbiReturn {
    fn from(error: Error) -> SbiReturn {
        SbiReturn {
            error_code: error as i64,
            return_value: 0,
        }
    }
}

impl From<SbiReturn> for Result<u64> {
    fn from(ret: SbiReturn) -> Result<u64> {
        match ret.error_code {
            SBI_SUCCESS => Ok(ret.return_value),
            e => Err(Error::from_code(e)),
        }
    }
}

/// SBI return value conventions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SbiReturnType {
    /// Legacy (v0.1) extensions return a single value in A0, usually with the convention that 0
    /// is success and < 0 is an implementation defined error code.
    Legacy(u64),
    /// Modern extensions use the standard error code values enumerated above.
    Standard(SbiReturn),
}

/// SBI Message used to invoke the specified SBI extension in the firmware.
#[derive(Clone, Copy)]
pub enum SbiMessage {
    /// The base SBI extension functions.
    Base(BaseFunction),
    /// The legacy PutChar extension.
    PutChar(u64),
    /// The extension for getting/setting the state of CPUs.
    HartState(StateFunction),
    /// Handles system reset.
    Reset(ResetFunction),
    /// Provides capabilities for starting confidential virtual machines.
    Tee(TeeFunction),
    /// The extension for getting attestation evidences and extending measurements.
    Attestation(AttestationFunction),
}

impl SbiMessage {
    /// Creates an SbiMessage struct from the given GPRs. Intended for use from the ECALL handler
    /// and passed the saved register state from the calling OS. A7 must contain a valid SBI
    /// extension and the other A* registers will be interpreted based on the extension A7 selects.
    pub fn from_regs(args: &[u64]) -> Result<Self> {
        match args[7] {
            EXT_PUT_CHAR => Ok(SbiMessage::PutChar(args[0])),
            EXT_BASE => BaseFunction::from_regs(args).map(SbiMessage::Base),
            EXT_HART_STATE => StateFunction::from_regs(args).map(SbiMessage::HartState),
            EXT_RESET => ResetFunction::from_regs(args).map(SbiMessage::Reset),
            EXT_TEE => TeeFunction::from_regs(args).map(SbiMessage::Tee),
            EXT_ATTESTATION => AttestationFunction::from_regs(args).map(SbiMessage::Attestation),
            _ => Err(Error::NotSupported),
        }
    }

    /// Returns the register value for this `SbiMessage`.
    pub fn a7(&self) -> u64 {
        match self {
            SbiMessage::Base(_) => EXT_BASE,
            SbiMessage::PutChar(_) => EXT_PUT_CHAR,
            SbiMessage::HartState(_) => EXT_HART_STATE,
            SbiMessage::Reset(_) => EXT_RESET,
            SbiMessage::Tee(_) => EXT_TEE,
            SbiMessage::Attestation(_) => EXT_ATTESTATION,
        }
    }

    /// Returns the register value for this `SbiMessage`.
    pub fn a6(&self) -> u64 {
        match self {
            SbiMessage::Base(_) => 0, //TODO
            SbiMessage::HartState(f) => f.a6(),
            SbiMessage::PutChar(_) => 0,
            SbiMessage::Reset(_) => 0,
            SbiMessage::Tee(f) => f.a6(),
            SbiMessage::Attestation(f) => f.a6(),
        }
    }

    /// Returns the register value for this `SbiMessage`.
    pub fn a5(&self) -> u64 {
        match self {
            SbiMessage::Tee(f) => f.a5(),
            _ => 0,
        }
    }

    /// Returns the register value for this `SbiMessage`.
    pub fn a4(&self) -> u64 {
        match self {
            SbiMessage::Tee(f) => f.a4(),
            _ => 0,
        }
    }

    /// Returns the register value for this `SbiMessage`.
    pub fn a3(&self) -> u64 {
        match self {
            SbiMessage::Tee(f) => f.a3(),
            SbiMessage::Attestation(f) => f.a3(),
            _ => 0,
        }
    }

    /// Returns the register value for this `SbiMessage`.
    pub fn a2(&self) -> u64 {
        match self {
            SbiMessage::HartState(f) => f.a2(),
            SbiMessage::Tee(f) => f.a2(),
            SbiMessage::Attestation(f) => f.a2(),
            _ => 0,
        }
    }

    /// Returns the register value for this `SbiMessage`.
    pub fn a1(&self) -> u64 {
        match self {
            SbiMessage::Reset(r) => r.a1(),
            SbiMessage::HartState(f) => f.a1(),
            SbiMessage::Tee(f) => f.a1(),
            SbiMessage::Attestation(f) => f.a1(),
            _ => 0,
        }
    }

    /// Returns the register value for this `SbiMessage`.
    pub fn a0(&self) -> u64 {
        match self {
            SbiMessage::Reset(r) => r.a0(),
            SbiMessage::PutChar(c) => *c,
            SbiMessage::HartState(f) => f.a0(),
            SbiMessage::Tee(f) => f.a0(),
            SbiMessage::Attestation(f) => f.a0(),
            _ => 0,
        }
    }

    /// Returns the result returned in the SbiMessage. Intended for use after an SbiMessage has been
    /// handled by the firmware. Interprets the given registers based on the extension and function
    /// and returns the approprate result.
    ///
    /// # Example
    ///
    /// ```rust
    /// pub fn ecall_send(msg: &SbiMessage) -> Result<u64> {
    ///     let mut a0 = msg.a0(); // error code
    ///     let mut a1 = msg.a1(); // return value
    ///     unsafe {
    ///         // Safe, but relies on trusting the hypervisor or firmware.
    ///         asm!("ecall", inout("a0") a0, inout("a1")a1,
    ///                 in("a2")msg.a2(), in("a3") msg.a3(),
    ///                 in("a4")msg.a4(), in("a5") msg.a5(),
    ///                 in("a6")msg.a6(), in("a7") msg.a7());
    ///     }
    ///
    ///     msg.result(a0, a1)
    /// }
    /// ```
    pub fn result(&self, a0: u64, a1: u64) -> Result<u64> {
        let ret = SbiReturn {
            error_code: a0 as i64,
            return_value: a1,
        };
        match self {
            // For legacy messages, a0 is 0 on success and an implementation-defined error value on
            // failure. Nothing is returned in a1.
            SbiMessage::PutChar(_) => match a0 as i64 {
                SBI_SUCCESS => Ok(0),
                _ => Err(Error::Failed),
            },
            _ => ret.into(),
        }
    }
}

/// Send an ecall to the firmware or hypervisor.
///
/// # Safety
///
/// The caller must verify that any memory references contained in `msg` obey rust's memory
/// safety rules. For example, any pointers to memory that will be modified in the handling of
/// the ecall must be uniquely owned. Similarly any pointers read by the ecall must not be
/// mutably borrowed.
///
/// In addition the caller is placing trust in the firmware or hypervisor to maintain the promises
/// of the interface w.r.t. reading and writing only within the provided bounds.
#[cfg(all(target_arch = "riscv64", target_os = "none"))]
pub unsafe fn ecall_send(msg: &SbiMessage) -> Result<u64> {
    // normally error code
    let mut a0;
    // normally return value
    let mut a1;
    asm!("ecall", inlateout("a0") msg.a0()=>a0, inlateout("a1")msg.a1()=>a1,
                in("a2")msg.a2(), in("a3") msg.a3(),
                in("a4")msg.a4(), in("a5") msg.a5(),
                in("a6")msg.a6(), in("a7") msg.a7(), options(nostack));

    msg.result(a0, a1)
}
