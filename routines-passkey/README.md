```rs,no_run
Registration
────────────
Browser        Axum Server
   │                │
   │ Begin Register │
   ├───────────────►│
   │ Challenge      │
   │◄───────────────┤
   │ navigator.credentials.create()
   │                │
   │ Attestation    │
   ├───────────────►│
   │                │ Verify
   │                │ Store public key
   │                │


Authentication
──────────────
Browser        Axum Server
   │                │
   │ Begin Login    │
   ├───────────────►│
   │ Challenge      │
   │◄───────────────┤
   │ navigator.credentials.get()
   │                │
   │ Assertion      │
   ├───────────────►│
   │                │ Verify signature
   │                │ Create session
```
