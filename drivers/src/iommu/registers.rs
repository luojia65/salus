// Copyright (c) 2022 by Rivos Inc.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use assertions::const_assert;
use tock_registers::register_bitfields;
use tock_registers::registers::{ReadOnly, ReadWrite};

// IOMMU register definitions; see https://github.com/riscv-non-isa/riscv-iommu.

register_bitfields![u64,
    pub Capabilities [
        Version OFFSET(0) NUMBITS(8),
        Sv32 OFFSET(8) NUMBITS(1),
        Sv39 OFFSET(9) NUMBITS(1),
        Sv48 OFFSET(10) NUMBITS(1),
        Sv57 OFFSET(11) NUMBITS(1),
        Sv32x4 OFFSET(16) NUMBITS(1),
        Sv39x4 OFFSET(17) NUMBITS(1),
        Sv48x4 OFFSET(18) NUMBITS(1),
        Sv57x4 OFFSET(19) NUMBITS(1),
        MsiFlat OFFSET(22) NUMBITS(1),
        MsiMrif OFFSET(23) NUMBITS(1),
    ],
];

/// The IOMMU register set.
#[repr(C)]
pub struct IommuRegisters {
    pub capabilities: ReadOnly<u64, Capabilities::Register>,
    pub fctrl: ReadWrite<u32>,
    _reserved0: u32,
    pub ddtp: ReadWrite<u64>,
    pub cqb: ReadWrite<u64>,
    pub cqh: ReadWrite<u32>,
    pub cqt: ReadOnly<u32>,
    pub fqb: ReadWrite<u64>,
    pub fqh: ReadWrite<u32>,
    pub fqt: ReadOnly<u32>,
    pub pqb: ReadWrite<u64>,
    pub pqh: ReadWrite<u32>,
    pub pqt: ReadOnly<u32>,
    pub cqcsr: ReadWrite<u32>,
    pub fqcsr: ReadWrite<u32>,
    pub pqcsr: ReadWrite<u32>,
    pub ipsr: ReadWrite<u32>,
    // Includes debug/performance counter registers which we don't care about at the moment.
    _reserved1: [u32; 1002],
}

fn _assert_register_layout() {
    const_assert!(core::mem::size_of::<IommuRegisters>() == 4096);
}