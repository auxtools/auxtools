# Temporary savefiles clash under Wine when worlds start in the same second

Worlds started in the same second give their temporary savefiles the same name. Under Wine,
`new /savefile` fails with a runtime error that has no name while another world has that file open,
and the test suites run several worlds at once.

To avoid this clash, the `auxtest` list samples open a savefile named after the world's process,
`auxtest_[world.process].sav`, deleting any existing file of that name first and deleting it again
when closing it.

## Applies to

- `tests/auxtest_host/auxtest_host.dm`: `auxtest_open_savefile`, `auxtest_close_savefile`
