@echo off
cd %*
if "%NVC_VERSION_FILE_STRATEGY%" == "recursive" (
  nvc use --silent-if-unchanged
) else (
  if exist .nvmrc (
    nvc use --silent-if-unchanged
  ) else (
    if exist .node-version (
      nvc use --silent-if-unchanged
    )
  )
)
@echo on
