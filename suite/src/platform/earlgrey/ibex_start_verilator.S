/**
 * ROM_EXT Interrupt Vector, adapted from opentitan/sw/device/silicon_owner/bare_metal/bare_metal_start.S
 */
  .section .vectors, "ax"
  
  .option push
  .option norvc
  .option norelax

  /**
   * Ibex-compatible interrupt vector.
   *
   * For more information see:
   * https://ibex-core.readthedocs.io/en/latest/03_reference/exception_interrupts.html
   */
  .balign 256
  .global _interrupt_vector
  .type _interrupt_vector, @function
_interrupt_vector:
  // Exception and User Software Interrupt Handler.
  unimp
  // Supervisor Software Interrupt Handler.
  unimp
  // Reserved.
  unimp
  // Machine Software Interrupt Handler.
  unimp

  // User Timer Interrupt Handler.
  unimp
  // Supervisor Timer Interrupt Handler.
  unimp
  // Reserved.
  unimp
  // Machine Timer Interrupt Handler.
  unimp

  // User External Interrupt Handler.
  unimp
  // Supervisor External Interrupt Handler.
  unimp
  // Reserved.
  unimp
  // Machine External Interrupt Handler.
  unimp

  // Reserved.
  unimp
  unimp
  unimp
  unimp

  // On Ibex interrupt ids 16-30 are for "fast" interrupts.
  .rept 15
  unimp
  .endr

  // On Ibex interrupt id 31 is for non-maskable interrupts.
  unimp
  .size _interrupt_vector, .-_interrupt_vector

  .option pop

// ---------------------------------------------------------------------------

/**
 * Opentitan Manifest information
 *
 * For more information see:
 * https://docs.opentitan.org/sw/device/silicon_creator/rom_ext/docs/manifest/
 */

  .section .text.manifest

  .extern _start

_opentitan_manifest:

  // For the testing ROM only the entry_point is relevant so ignore the prior fields 
  .skip 892
  .word _start
