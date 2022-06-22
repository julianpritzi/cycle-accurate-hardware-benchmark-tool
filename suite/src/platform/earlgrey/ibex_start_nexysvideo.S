/**
 * Opentitan Manifest information
 *
 * For more information see:
 * https://docs.opentitan.org/sw/device/silicon_creator/rom_ext/docs/manifest/
 */

  .section .text.manifest

  .extern _start

_opentitan_manifest:

  // For the testing ROM only the entry_point is relevant, this older version only has the entry point field
  .word _start
