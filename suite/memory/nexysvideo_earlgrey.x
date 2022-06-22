/* memory layout of the opentitan earlgrey chip
*  For more information see:
*  https://docs.opentitan.org/hw/top_earlgrey/doc/#register-table
*/

MEMORY
{
  SRAM(w) : ORIGIN = 0x10000000, LENGTH = 0x10000
  Flash(rx) : ORIGIN = 0x20000000, LENGTH = 0x100000
}

REGION_ALIAS("REGION_TEXT", Flash);
REGION_ALIAS("REGION_RODATA", Flash);
REGION_ALIAS("REGION_DATA", SRAM);
REGION_ALIAS("REGION_BSS", SRAM);
REGION_ALIAS("REGION_HEAP", SRAM);
REGION_ALIAS("REGION_STACK", SRAM);

ENTRY(_start)

/* We have to alter the default riscv-rt linker script because it does not work for the Ibex core */
SECTIONS
{
  /* Custom Opentitan Manifest for the test rom */
  .text.manifest :
  {
    KEEP(*(.text.manifest));
  } > REGION_TEXT

  .text.dummy (NOLOAD) :
  {
    /* This section is intended to make _stext address work */
    _stext = .;
  } > REGION_TEXT
}

_heap_size = 4K;
