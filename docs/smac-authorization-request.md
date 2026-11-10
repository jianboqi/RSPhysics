# SMAC Apache-2.0 permission inquiry draft

This is a draft for the maintainer to send, not a sent message or an authorization
already obtained. Contact Olivier Hagolle and ask whether other code/coefficient
rights holders also need to consent. Use public contact information or Issues at
the [author repository](https://github.com/olivierhagolle/SMAC); no private email
address is guessed here.

## 📘 Message draft

Subject:Permission to redistribute a SMAC Rust adaptation and coefficients under Apache-2.0

Dear Olivier,

We are developing RSPhysics, an open-source remote-sensing physics library
with a shared Rust scientific kernel, a Python package, and a browser/WASM
application. We would like to distribute our original project code under
the Apache License 2.0 and include SMAC with its scientific citations and
CNES acknowledgements preserved.

Our current SMAC adaptation and numerical reference use the implementation
preserved in ESA Sen2Like, commit
`6dd3f0da82a7461b03a2914df56d305070e50d6d`, under
`sen2like/sen2like/atmcor/smac/`. Its `copyright.txt` states GPL v3.
We also inspected your SMAC repository at commit
`77bf73dd1f2a2b6a33a97b8997da469210f9ce3e`, where we did not find
an explicit license file.

Could you please clarify the applicable licensing and, if you have the
necessary authority, grant permission to use, modify and redistribute the
relevant code and our adaptation under Apache-2.0, including sublicensing
to downstream recipients on those terms?

Please also clarify whether that permission can cover the coefficient data:

- The legacy sensor coefficient files in your repository's `COEFS` directory.
- The Landsat-8 OLI and Sentinel-2A/B/C continental coefficient files in the
  fixed Sen2Like copy above, including the multi-band tables generated for CNES.
- Lossless repackaging, redistribution and use of those coefficients within
  Python packages, WASM files and commercial or noncommercial applications.

If some code or data belongs to other rights holders, could you identify
the materials and the additional permissions we should obtain? We understand
that a code permission does not automatically grant rights to third-party data.

An explicit statement identifying the covered materials/versions and granting
Apache-2.0 rights, or a corresponding upstream license update, would help us
preserve an accurate authorization record. We will retain the Rahman–Dedieu
paper citation, author credits, and CNES acknowledgements.

Thank you for your work on SMAC and for helping us clarify these terms.

Best regards,
RSPhysics maintainer

## 📘 Recording a response

Record the response date, authorizer and authority, covered commits/files, and
separate code/data conditions. A general permission to "use" or "open-source"
material does not by itself establish complete Apache-2.0 redistribution and
downstream licensing rights; request clarification when needed. Confirm what
may be made public before publishing a reply, name, email text, or attachments.
Do not place private contact information in the repository.
