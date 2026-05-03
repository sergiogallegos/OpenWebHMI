# TwinCAT 3 ADS Smoke Project

This directory records the minimal PLC surface used by the Beckhoff TwinCAT 3 manual smoke runbook in `apps/designer/README.md`.

Exporting a portable `.tsproj` from TwinCAT XAE is environment-specific because project files include generated solution metadata and target mappings. For the smoke test, create a TwinCAT 3 PLC project with the following `MAIN.PRG` program and activate it on runtime port `851`.

```iecst
PROGRAM MAIN
VAR
    bRunning : BOOL := TRUE;
    nCounter : INT := 0;
    fSetPoint : REAL := 12.5;
    sStatus : STRING(80) := 'OpenWebHMI ADS smoke';
END_VAR

IF bRunning THEN
    nCounter := nCounter + 1;
END_IF
```

Expected OpenWebHMI ADS tag addresses:

- `851:MAIN.bRunning`
- `851:MAIN.nCounter`
- `851:MAIN.fSetPoint`
- `851:MAIN.sStatus`

Use `backend: "auto"` on Windows with TwinCAT installed so XAE-created Secure ADS routes go through Beckhoff `TcAdsDll.dll`. Use `backend: "ads_rs_tcp"` only for plain ADS-over-TCP routes.
